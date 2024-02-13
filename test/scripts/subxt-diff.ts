import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { spawn, exec, ChildProcessWithoutNullStreams } from "child_process";
import { setTimeout } from "timers/promises";
import { promisify } from "util";
import { fileURLToPath } from "url";
import path from "path";
import fs from "fs/promises";
import { Octokit } from "octokit";

const execPromise = promisify(exec);
const octokit = new Octokit();
const Runtimes = ["moonbeam", "moonriver", "moonbase"] as const;
type RuntimeType = (typeof Runtimes)[number];

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Github API Functions
const getLatestRTRelease = async () => {
  try {
    const response = await octokit.rest.repos.listReleases({
      owner: "moonbeam-foundation",
      repo: "moonbeam",
    });

    const rtReleases = response.data.filter(
      ({ tag_name, draft }) => tag_name.includes("runtime") && !draft
    );

    return rtReleases[0];
  } catch (e) {
    console.error(e);
    throw new Error("Failed to fetch latest Runtime release from moonbeam-foundation/moonbeam");
  }
};

const getCommitFromTag = async (tag: string): Promise<string> => {
  try {
    const response = await octokit.rest.git.getRef({
      owner: "moonbeam-foundation",
      repo: "moonbeam",
      ref: `tags/${tag}`,
    });

    let fullSha: string | undefined;

    if (response.data.object.type === "commit") {
      fullSha = response.data.object.sha;
    } else if (response.data.object.type === "tag") {
      const tagResponse = await octokit.rest.git.getTag({
        owner: "moonbeam-foundation",
        repo: "moonbeam",
        tag_sha: response.data.object.sha,
      });
      fullSha = tagResponse.data.object.sha;
    }

    if (!fullSha) {
      throw new Error(`Failed to fetch commit from tag: ${tag}`);
    }

    return fullSha.slice(0, 8);
  } catch (e) {
    console.error(e);
    throw new Error(`Failed to fetch commit from tag: ${tag}`);
  }
};

// Node Helpers
interface RuntimeResponse {
  jsonrpc: string;
  result: RuntimeDetails;
  id: number;
}

const launchArgs = (runtime: string, port: number) => [
  `--chain=${runtime}-dev`,
  "--no-hardware-benchmarks",
  "--no-telemetry",
  "--reserved-only",
  "--rpc-cors=all",
  "--no-grandpa",
  "--sealing=manual",
  "--force-authoring",
  "--no-prometheus",
  "--unsafe-rpc-external",
  "--alice",
  `--rpc-port=${port}`,
  "--tmp",
];

interface RuntimeDetails {
  specName: string;
  implName: string;
  authoringVersion: number;
  specVersion: number;
  implVersion: number;
  apis: [string, number][];
  transactionVersion: number;
  stateVersion: number;
}

// misc
const writeDiffResultsToFile = async (runtime: string, runtimeVersion: number, content: string) => {
  const diffsDirPath = path.join(__dirname, "../../runtime-diffs", runtime);
  const fileName = `runtime-${runtimeVersion}.txt`;
  const filePath = path.join(diffsDirPath, fileName);
  // biome-ignore lint/suspicious/noControlCharactersInRegex: this is what subxt adds
  const cleanContent = content.replace(/\x1B\[[0-?]*[ -/]*[@-~]/g, "");
  try {
    await fs.mkdir(diffsDirPath, { recursive: true });
    await fs.writeFile(filePath, cleanContent);

    // Extract the file name
    const fileName = path.basename(filePath);
    const directoryPath = path.dirname(filePath);
    const parts = directoryPath.split(path.sep);
    const result = path.join(parts[parts.length - 2], parts[parts.length - 1], fileName);

    process.stdout.write(` ${result}  ✅\n`);
  } catch (error) {
    process.stdout.write("  ❌ \n");
    console.error(`Error writing diff results to file: ${error}`);
  }
};

// CLI - Main Code
yargs(hideBin(process.argv))
  .usage("Usage: $0")
  .version("1.0.0")
  .command<{ runtime: RuntimeType; publish: boolean }>(
    "diff <runtime> <releaseSha8> [publish]",
    "Runs a diff between metadata from current runtime and last release",
    (yargs) => {
      return yargs
        .positional("runtime", {
          describe: "The runtime to compare",
          type: "string",
          choices: Runtimes,
        })
        .positional("releaseSha8", {
          describe: "The sha8 of the release to compare",
          type: "string",
        })
        .positional("publish", {
          describe: "Raise a PR to commit the diff results to repo",
          type: "boolean",
          default: false,
        });
    },

    async (argv: { runtime: RuntimeType; publish: boolean }) => {
      let localNodeProcess: ChildProcessWithoutNullStreams | undefined;

      try {
        console.log(`🟢 Running diff for runtime: ${argv.runtime}`);

        const latestRelease = await getLatestRTRelease();
        const latestReleaseVersion = parseInt(latestRelease.tag_name.split("runtime-")[1]);
        console.log(`🟢 Latest release version: ${latestReleaseVersion}`);

        const nodePath = path.join(__dirname, "../../target/release/moonbeam");

        try {
          await fs.access(nodePath, fs.constants.R_OK | fs.constants.W_OK);
          console.log(`🟢 Can access node binary at ${nodePath}`);
        } catch (e) {
          console.error(e);
          console.error(`❌ Cannot access ${nodePath}`);
          throw new Error("Failed to access node binary");
        }

        try {
          localNodeProcess = spawn(nodePath, launchArgs(argv.runtime, 9977), { shell: true });
          console.log("🟢 Local moonbeam node spawned");

          localNodeProcess.stderr.on("data", (data) => {
            console.log(data);
          });
        } catch (e) {
          console.error(e);
          throw new Error("Failed to spawn local node process");
        }

        localNodeProcess.on("close", (code) => {
          process.exit(code ? code : 0); // Exit with the child process's exit code
        });

        await setTimeout(2000);

        const headers = {
          "Content-Type": "application/json",
        };

        const body = JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "state_getRuntimeVersion",
          params: [],
        });

        let localRuntimeVersion: number;

        try {
          const response = await fetch("http://127.0.0.1:9977", {
            method: "POST",
            headers: headers,
            body: body,
          });

          const runtimeDetails = (await response.json()) as RuntimeResponse;
          localRuntimeVersion = runtimeDetails.result.specVersion;
        } catch (e) {
          console.error(e);
          throw new Error("Failed to fetch runtime details from local node");
        }

        console.log(`🆕 Local runtime version: ${localRuntimeVersion}`);

        const sha8 = await getCommitFromTag(latestRelease.tag_name);
        console.log(`🔗 sha8 from runtime ${latestRelease.tag_name}: ${sha8}`);

        const { stdout } = await execPromise(
          "subxt diff -a ws://127.0.0.1:9977 ws://127.0.0.1:9911"
        );
        console.log("🔎 Diff Results:");
        console.log(stdout);

        if (argv.publish) {
          process.stdout.write("📝 Writing diff to file...");
          await writeDiffResultsToFile(argv.runtime, localRuntimeVersion, stdout);
        }
      } catch (e) {
        console.error(e);
        process.exitCode = 1;
      } finally {
        if (localNodeProcess) {
          localNodeProcess.kill();
        }
      }
    }
  )
  .strictCommands()
  .demandCommand(1)
  .parse();
