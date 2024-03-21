import fs from "fs/promises";
import path from "path";
import child_process from "child_process";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("2.0.0")
  .options({
    OutputDirectory: {
      type: "string",
      alias: "o",
      description: "Output directory for compiled contracts",
      default: "precompiled-wasm",
    },
    Binary: {
      type: "string",
      alias: "b",
      description: "Moonbeam binary path",
      default: "contracts/src",
    },
    Chain: {
      type: "string",
      alias: "c",
      description: "runtime chain to use",
      require: true,
    },
    Verbose: {
      type: "boolean",
      alias: "v",
      description: "Verbose mode for extra logging.",
      default: false,
    },
  })
  .command("compile", "Compile wasm", async (argv) => {
    await main(argv as any);
  })
  .parse();

async function spawn(cmd: string) {
  return new Promise((resolve, reject) => {
    var spawned = child_process.spawn(cmd, { shell: true });

    let errData = "";
    let outData = "";
    spawned.stdout.on("data", (chunk) => {
      outData += chunk.toString();
    });

    spawned.stderr.on("data", (chunk) => {
      errData += chunk.toString();
    });

    spawned.on("close", function (code) {
      if (code && code > 0) {
        return reject(new Error(errData));
      }

      resolve(outData);
    });

    spawned.on("error", function (err) {
      reject(err);
    });
  });
}

async function main(args: any) {
  const outputDirectory = path.join(process.cwd(), args.argv.OutputDirectory);
  const binaryPath = args.argv.Binary;

  console.log(`🗃️  Binary: ${binaryPath}`);
  console.log(`🗃️  Output directory: ${outputDirectory}`);

  child_process.execSync(`mkdir -p ${outputDirectory}`);

  await fs.mkdir("tmp", { recursive: true });
  const tmpDir = await fs.mkdtemp("tmp/base-path");
  try {
    // Generate plain chain spec
    const generateChainSpecCmd =
      `${binaryPath} build-spec` + `--chain ${args.argv.Chain} > tmp/${args.argv.Chain}.json`;
    console.log(`🗃️  ${generateChainSpecCmd}`);
    await spawn(generateChainSpecCmd);

    // Generate raw chain spec
    const generateRawChainSpecCmd =
      `${binaryPath} build-spec --chain tmp/${args.argv.Chain}.json ` +
      `--raw > tmp/${args.argv.Chain}-raw.json`;
    console.log(`🗃️  ${generateRawChainSpecCmd}`);
    await spawn(generateRawChainSpecCmd);

    // Generate precompiled wasm
    const command =
      `${binaryPath} precompile-wasm --log=wasmtime-runtime --base-path=${tmpDir} ` +
      `--chain tmp/${args.argv.Chain}-raw.json ${outputDirectory}`;
    console.log(`🗃️  ${command}`);
    await spawn(command);
  } finally {
    if ((await fs.stat(tmpDir)).isDirectory()) {
      await fs.rm(tmpDir, { recursive: true, force: true });
    }
  }
}
