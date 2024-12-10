#!/usr/bin/env ts-node

import { exec as execProcess } from "node:child_process";
import yargs from "yargs";
import util from "node:util";
import fs from "node:fs";
import os from "os";
import path from "path";
import { strict as assert } from "node:assert";

const exec = util.promisify(execProcess);

const openCmd = (() => {
  switch (process.platform) {
    case "darwin":
      return "open";
    case "win32":
      return "start";
    default:
      return "xdg-open";
  }
})();

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: $0")
    .version("1.0.0")
    .command("analyze", "analyze multiple PoV analysis", (yargs) => {
      yargs
        .option("input", {
          type: "string",
          describe: "Input JSON files",
          array: true,
        })
        .option("output", {
          type: "string",
          default: "output-analyze.html",
          describe: "The output HTML file",
        })
        .demandOption(["input"]);
    })
    .command("view", "view a PoV analysis", (yargs) => {
      yargs
        .option("input", {
          type: "string",
          describe: "Input JSON file",
        })
        .option("open", {
          type: "boolean",
          default: true,
          describe: "Open the file in default application",
        })
        .option("output", {
          type: "string",
          default: "output.html",
          describe: "The output HTML file",
        })
        .demandOption(["input"]);
    })
    .command("run", "run a given benchmark for PoV analysis", (yargs) => {
      yargs
        .option("pallet", {
          type: "string",
          describe: "A pallet name as snake_case",
        })
        .option("benchmark", {
          type: "string",
          describe: "The benchmark name",
        })
        .option("params", {
          type: "string",
          describe:
            "The parameter values. Example: '1 50 100' (single param) or \
            '1,1000 50,2000 100,3000' (multiple param)",
        })
        .option("output", {
          type: "string",
          default: "output.json",
          describe: "The output JOSN file",
        })
        .option("view", {
          type: "boolean",
          default: false,
          describe: "View the file in default application",
        })
        .demandOption(["pallet", "benchmark", "params"]);
    })
    .demandCommand()
    .strict().argv as any;

  const [command] = argv._;

  switch (command) {
    case "run":
      const runResults = await runAll(
        argv.pallet,
        argv.benchmark,
        (argv.params as string).split(" ").filter((x) => !!x)
      );

      fs.writeFileSync(argv.output, JSON.stringify(runResults));
      if (argv.view) {
        const tmpFile = path.join(os.tmpdir(), `${argv.output}.html`);
        await view(argv.output, tmpFile, true);
      }
      break;
    case "view":
      await view(argv.input, argv.output, argv.open);
      break;
    case "analyze":
      await analyze(argv.input, argv.output);
      break;
  }

  return;
}

type StorageInfo = { [key: string]: { [key: string]: { reads: number; writes: number } } };
type RunInfo = {
  pallet: string;
  benchmark: string;
  parameters: number[];
  extrinsicTime: number;
  storageRootTime: number;
  totalReads: number;
  totalWrites: number;
  totalRepeatReads: number;
  totalRepeatWrites: number;
  proofSize: number;
  storageInfo: StorageInfo;
};

async function runAll(pallet: string, benchmark: string, parameters: string[]): Promise<RunInfo[]> {
  const runResults: RunInfo[] = [];

  for (const parameter of parameters) {
    console.log(`running ${pallet}:${benchmark} for ${parameter}`);

    const params = parameter
      .split(",")
      .filter((x) => !!x)
      .map((n) => parseInt(n, 10));
    const low_high = params.join(",");

    const rawFilename = "raw-output.json";

    const binary = process.env["BINARY"] || "./target/release/moonbeam";
    const { stdout, stderr } = await exec(
      `${binary} benchmark pallet \
        --chain dev \
        --execution=wasm \
        --log error \
        --wasm-execution=compiled \
        --pallet "${pallet}" \
        --extrinsic "${benchmark}" \
        --no-median-slopes \
        --no-min-squares \
        --steps "1" \
        --repeat "1" \
        --low "${low_high}" \
        --high "${low_high}" \
        --record-proof \
        --json-file ${rawFilename}
    `,
      {
        env: {
          WASMTIME_BACKTRACE_DETAILS: "1",
        },
        maxBuffer: 5 * 1024 * 1024,
      }
    );

    if (!stdout) {
      throw new Error(`STDERR! ${stderr}`);
    }

    let startRead = false;
    const storageInfo: StorageInfo = {};
    const lines = stdout.split("\n");
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i].trim();
      if (line === "Raw Storage Info") {
        i++;
        startRead = true;
      } else if (startRead) {
        if (!line.length) {
          break;
        }

        const matches = line.match(/Storage: (\w+) (\w+) \(r:(\d+) w:(\d+)\)/);
        if (!matches) {
          throw new Error(`unable to match for storage info in "${line}"`);
        }

        const [_, storagePallet, storageItem, reads, writes] = matches;
        if (!storageInfo[storagePallet]) {
          storageInfo[storagePallet] = {};
        }

        storageInfo[storagePallet][storageItem] = {
          reads: parseInt(reads),
          writes: parseInt(writes),
        };
      }
    }

    const rawFile = JSON.parse(fs.readFileSync(rawFilename).toString("utf-8"));
    assert.equal(rawFile[0]["pallet"], pallet);
    assert.equal(rawFile[0]["benchmark"], benchmark);
    const dbResults = rawFile[0]["db_results"];

    const info = {
      pallet,
      benchmark,
      parameters: params,
      extrinsicTime: 0,
      storageRootTime: 0,
      totalReads: 0,
      totalWrites: 0,
      totalRepeatReads: 0,
      totalRepeatWrites: 0,
      proofSize: 0,
      storageInfo,
    };

    for (const dbResult of dbResults) {
      info.extrinsicTime += dbResult["extrinsic_time"];
      info.storageRootTime += dbResult["storage_root_time"];
      info.totalReads += dbResult["reads"];
      info.totalWrites += dbResult["writes"];
      info.totalRepeatReads += dbResult["repeat_reads"];
      info.totalRepeatWrites += dbResult["repeat_writes"];
      info.proofSize += dbResult["proof_size"];
    }

    info.extrinsicTime = Math.floor(info.extrinsicTime / dbResults.length);
    info.storageRootTime = Math.floor(info.storageRootTime / dbResults.length);
    info.totalReads = Math.floor(info.totalReads / dbResults.length);
    info.totalWrites = Math.floor(info.totalWrites / dbResults.length);
    info.totalRepeatReads = Math.floor(info.totalRepeatReads / dbResults.length);
    info.totalRepeatWrites = Math.floor(info.totalRepeatWrites / dbResults.length);
    info.proofSize = Math.floor(info.proofSize / dbResults.length);

    runResults.push(info);
  }

  return runResults;
}

async function view(input: string, output: string, open: boolean) {
  const data = JSON.parse(fs.readFileSync(input).toString("utf-8"));

  const labels = data.map((x: any) => x["parameters"].join(","));
  const proofSize = data.map((x: any) => x["proofSize"]);
  const totalReads = data.map((x: any) => x["totalReads"]);
  const totalWrites = data.map((x: any) => x["totalWrites"]);
  const extrinsicTime = data.map((x: any) => x["extrinsicTime"]);

  // editorconfig-checker-disable
  fs.writeFileSync(
    output,
    `<html>
    <head>
      <script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/3.7.1/chart.min.js" integrity="sha512-QSkVNOCYLtj73J4hbmVoOV6KVZuMluZlioC+trLpewV8qMjsWqlIQvkn1KGX2StWvPMdWGBqim1xlC8krl1EKQ==" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
      <style>
        .chart {
          display: inline-block;
          width: 1000px;
          height: 600px;
          margin: 10px;
        }
      </style>
    </head>
    <body>
      <div class="chart">
        <canvas id="proof-size"></canvas>
      </div>  
      <div class="chart">
        <canvas id="reads-writes"></canvas>
      </div>  
      <div class="chart">
        <canvas id="extrinsic-time"></canvas>
      </div>
      <script>
        const rawData = ${JSON.stringify(data)};
        function flattenStorageInfo(si) {
          const info = [];
          Object.entries(si).forEach(([pallet, value]) => {
            Object.entries(value).forEach(([storage, rw]) => {
              info.push(pallet + " " + storage + " r:"+rw.reads + " w:"+rw.writes);
            });
          });
          return info.join("\\n");
        }
        const footerReadsWrites = (tooltipItems) => flattenStorageInfo(rawData[tooltipItems[0].dataIndex]["storageInfo"]);

        const proofSize = new Chart(
          document.getElementById('proof-size').getContext('2d'), 
          {
            type: 'line',
            data: {
              labels: ${JSON.stringify(labels)},
              datasets:[
                {
                  label: "PoV Size",
                  data: ${JSON.stringify(proofSize)},
                  fill: false,
                  borderColor: "rgb(75, 192, 192)",
                  tension: 0.1,
                },
              ]
            },
            options: {
              responsive: true,
              scales: {
                x: {
                  title: {
                    display: true,
                    text: "Parameter Groups",
                    font: { weight: "bold" }
                  }
                },
                y: {
                  title: {
                    display: true,
                    text: "Size (bytes)",
                    font: { weight: "bold" }
                  }
                }
              },
              plugins: {
                legend: {
                  position: 'top',
                },
                title: {
                  display: true,
                  text: 'Proof Size'
                },
              },
            },
          });

        const readWrite = new Chart(
          document.getElementById('reads-writes').getContext('2d'), 
          {
            type: 'line',
            data: {
              labels: ${JSON.stringify(labels)},
              datasets:[
                {
                  label: "Reads",
                  data: ${JSON.stringify(totalReads)},
                  fill: false,
                  borderColor: "rgb(192, 75, 192)",
                  tension: 0.1,
                },
                {
                  label: "Writes",
                  data: ${JSON.stringify(totalWrites)},
                  fill: false,
                  borderColor: "rgb(192, 192, 75)",
                  tension: 0.1,
                }
              ]
            },
            options: {
              responsive: true,
              scales: {
                x: {
                  title: {
                    display: true,
                    text: "Parameter Groups",
                    font: { weight: "bold" }
                  }
                },
                y: {
                  title: {
                    display: true,
                    text: "Counts",
                    font: { weight: "bold" }
                  }
                }
              },
              plugins: {
                legend: {
                  position: 'top',
                },
                title: {
                  display: true,
                  text: 'Read/Writes'
                },
                tooltip: {
                  callbacks: {
                    footer: footerReadsWrites,
                  }
                },
              },
            },
          });
        
        const extrinsicTime = new Chart(
          document.getElementById('extrinsic-time').getContext('2d'), 
          {
            type: 'line',
            data: {
              labels: ${JSON.stringify(labels)},
              datasets:[{
                  label: "Extrinsic Time",
                  data: ${JSON.stringify(extrinsicTime)},
                  fill: false,
                  borderColor: "rgb(75, 75, 192)",
                  tension: 0.1,
                }]
            },
            options: {
              responsive: true,
              scales: {
                x: {
                  title: {
                    display: true,
                    text: "Parameter Groups",
                    font: { weight: "bold" }
                  }
                },
                y: {
                  title: {
                    display: true,
                    text: "Time (nanoseconds)",
                    font: { weight: "bold" }
                  }
                }
              },
              plugins: {
                legend: {
                  position: 'top',
                },
                title: {
                  display: true,
                  text: 'Extrinsic Time'
                },
              },
            },
          });

      </script>
    <body>
  </html>`
  );
  // editorconfig-checker-enable

  if (open) {
    await exec(`${openCmd} ${output}`);
  }
}

async function analyze(inputs: string[], output: string) {
  const dataMultiple = inputs.map((input) => JSON.parse(fs.readFileSync(input).toString("utf-8")));

  const labels = dataMultiple[0].map((x: any) => x["parameters"].join(","));
  const proofSizeMultiple = dataMultiple.map((data: any) => data.map((x: any) => x["proofSize"]));
  const totalReadsMultiple = dataMultiple.map((data: any) => data.map((x: any) => x["totalReads"]));
  const totalWritesMultiple = dataMultiple.map((data: any) =>
    data.map((x: any) => x["totalWrites"])
  );
  const extrinsicTimeMultiple = dataMultiple.map((data: any) =>
    data.map((x: any) => x["extrinsicTime"])
  );

  function random_rgb() {
    const o = Math.round;
    const r = Math.random;
    const s = 255;
    return `rgba(${o(r() * s)},${o(r() * s)},${o(r() * s)},1.0)`;
  }
  const colors = new Array(inputs.length).fill(0).map((x) => random_rgb());

  // editorconfig-checker-disable
  fs.writeFileSync(
    output,
    `<html>
    <head>
      <script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/3.7.1/chart.min.js" integrity="sha512-QSkVNOCYLtj73J4hbmVoOV6KVZuMluZlioC+trLpewV8qMjsWqlIQvkn1KGX2StWvPMdWGBqim1xlC8krl1EKQ==" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
      <style>
        .chart {
          display: inline-block;
          width: 1000px;
          height: 600px;
          margin: 10px;
        }
      </style>
    </head>
    <body>
      <div class="chart">
        <canvas id="proof-size"></canvas>
      </div>  
      <div class="chart">
        <canvas id="reads-writes"></canvas>
      </div>  
      <div class="chart">
        <canvas id="extrinsic-time"></canvas>
      </div>
      <script>
        /*
        ${JSON.stringify(colors, null, 2)}
        ${JSON.stringify(proofSizeMultiple, null, 2)}
        */
        const rawData = ${JSON.stringify(dataMultiple)};

        const proofSize = new Chart(
          document.getElementById('proof-size').getContext('2d'), 
          {
            type: 'line',
            data: {
              labels: ${JSON.stringify(labels)},
              datasets: ${JSON.stringify(
                proofSizeMultiple.map((p, i) => ({
                  label: inputs[i],
                  data: p,
                  fill: false,
                  borderColor: colors[i],
                  tension: 0.1,
                }))
              )}
            },
            options: {
              responsive: true,
              scales: {
                x: {
                  title: {
                    display: true,
                    text: "Parameter Groups",
                    font: { weight: "bold" }
                  }
                },
                y: {
                  title: {
                    display: true,
                    text: "Size (bytes)",
                    font: { weight: "bold" }
                  }
                }
              },
              plugins: {
                legend: {
                  position: 'top',
                },
                title: {
                  display: true,
                  text: 'Proof Size'
                },
              },
            },
          });

          function flattenStorageInfo(si) {
            const info = [];
            Object.entries(si).forEach(([pallet, value]) => {
              Object.entries(value).forEach(([storage, rw]) => {
                info.push(pallet + " " + storage + " r:"+rw.reads + " w:"+rw.writes);
              });
            });
            return info.join("\\n");
          }
          const footerReadsWrites = (tooltipItems) => {
            const inputIndex = tooltipItems[0].datasetIndex % ${inputs.length};
            return flattenStorageInfo(rawData[inputIndex][tooltipItems[0].dataIndex]["storageInfo"])
          };
          const readWrite = new Chart(
            document.getElementById('reads-writes').getContext('2d'), 
            {
              type: 'line',
              data: {
                labels: ${JSON.stringify(labels)},
                datasets: ${JSON.stringify([
                  ...totalReadsMultiple.map((p, i) => ({
                    label: `Reads - ${inputs[i]}`,
                    data: p,
                    fill: false,
                    borderColor: colors[i].replace("1.0", "0.5"),
                    tension: 0.1,
                    index: i,
                  })),
                  ...totalWritesMultiple.map((p, i) => ({
                    label: `Writes - ${inputs[i]}`,
                    data: p,
                    fill: false,
                    borderColor: colors[i],
                    borderDash: [5, 5],
                    tension: 0.1,
                    index: i,
                  })),
                ])}
              },
              options: {
                responsive: true,
                scales: {
                  x: {
                    title: {
                      display: true,
                      text: "Parameter Groups",
                      font: { weight: "bold" }
                    }
                  },
                  y: {
                    title: {
                      display: true,
                      text: "Counts",
                      font: { weight: "bold" }
                    }
                  }
                },
                plugins: {
                  legend: {
                    position: 'top',
                  },
                  title: {
                    display: true,
                    text: 'Read/Writes'
                  },
                  tooltip: {
                    callbacks: {
                      footer: footerReadsWrites,
                    }
                  },
                },
              },
            });

          const extrinsicTime = new Chart(
            document.getElementById('extrinsic-time').getContext('2d'), 
            {
              type: 'line',
              data: {
                labels: ${JSON.stringify(labels)},
                datasets: ${JSON.stringify(
                  extrinsicTimeMultiple.map((p, i) => ({
                    label: inputs[i],
                    data: p,
                    fill: false,
                    borderColor: colors[i],
                    tension: 0.1,
                  }))
                )}
              },
              options: {
                responsive: true,
                scales: {
                  x: {
                    title: {
                      display: true,
                      text: "Parameter Groups",
                      font: { weight: "bold" }
                    }
                  },
                  y: {
                    title: {
                      display: true,
                      text: "Time (nanoseconds)",
                      font: { weight: "bold" }
                    }
                  }
                },
                plugins: {
                  legend: {
                    position: 'top',
                  },
                  title: {
                    display: true,
                    text: 'Extrinsic Time'
                  },
                },
              },
            });
      </script>
    <body>
  </html>`
  );
  // editorconfig-checker-enable

  await exec(`${openCmd} ${output}`);
}

main().catch((err) => console.error(`ERR! ${err}`));
