import Web3 from "web3";

import { spawn, ChildProcess } from "child_process";
import { DISPLAY_LOG, PORT, RPC_PORT, SPAWNING_TIME, WS_PORT } from "../constants";
import { customRequest } from "./web3Requests";

const SPECS_PATH = `./moonbeam-test-specs`;

const FRONTIER_LOG = process.env.FRONTIER_LOG || "info";

const BINARY_PATH = `../../frontier/target/debug/frontier-template-node`;

// Build with `cargo build --no-default-features --features=manual-seal`
export async function startFrontierNode(
  specFilename: string,
  provider?: string
): Promise<{ web3: Web3; binary: ChildProcess }> {
  var web3;
  if (!provider || provider == "http") {
    web3 = new Web3(`http://localhost:${RPC_PORT}`);
  }

  const cmd = BINARY_PATH;
  const args = [
    `--chain=${SPECS_PATH}/${specFilename}`,
    `--validator`, // Required by manual sealing to author the blocks
    `--execution=Native`, // Faster execution using native
    `--no-telemetry`,
    `--no-prometheus`,
    `--sealing=Manual`,
    `--no-grandpa`,
    `--force-authoring`,
    `-l${FRONTIER_LOG}`,
    `--port=${PORT}`,
    `--rpc-port=${RPC_PORT}`,
    `--ws-port=${WS_PORT}`,
    `--tmp`,
  ];
  const binary = spawn(cmd, args);

  binary.on("error", (err) => {
    if ((err as any).errno == "ENOENT") {
      console.error(
        `\x1b[31mMissing Frontier binary (${BINARY_PATH}).\nPlease compile the Frontier project:\ncargo build\x1b[0m`
      );
    } else {
      console.error(err);
    }
    process.exit(1);
  });

  const binaryLogs = [];
  await new Promise<void>((resolve) => {
    const timer = setTimeout(() => {
      console.error(`\x1b[31m Failed to start Frontier Template Node.\x1b[0m`);
      console.error(`Command: ${cmd} ${args.join(" ")}`);
      console.error(`Logs:`);
      console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
      process.exit(1);
    }, SPAWNING_TIME - 2000);

    const onData = async (chunk) => {
      if (DISPLAY_LOG) {
        console.log(chunk.toString());
      }
      binaryLogs.push(chunk);
      if (chunk.toString().match(/Manual Seal Ready/)) {
        if (!provider || provider == "http") {
          // This is needed as the EVM runtime needs to warmup with a first call
          await web3.eth.getChainId();
        }

        clearTimeout(timer);
        if (!DISPLAY_LOG) {
          binary.stderr.off("data", onData);
          binary.stdout.off("data", onData);
        }
        // console.log(`\x1b[31m Starting RPC\x1b[0m`);
        resolve();
      }
    };
    binary.stderr.on("data", onData);
    binary.stdout.on("data", onData);
  });

  if (provider == "ws") {
    web3 = new Web3(`ws://localhost:${WS_PORT}`);
  }

  return { web3, binary };
}
export function describeWithFrontier(
  title: string,
  specFilename: string,
  cb: (context: { web3: Web3 }) => void,
  provider?: string
) {
  describe(title, () => {
    let context: { web3: Web3 } = { web3: null };
    let binary: ChildProcess;
    // Making sure the Frontier node has started
    before("Starting Frontier Test Node", async function () {
      this.timeout(SPAWNING_TIME);
      const init = await startFrontierNode(specFilename, provider);
      context.web3 = init.web3;
      binary = init.binary;
    });

    after(async function () {
      //console.log(`\x1b[31m Killing RPC\x1b[0m`);
      binary ? binary.kill() : null;
    });

    cb(context);
  });
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlockWithFrontier(web3: Web3) {
  const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
  if (!response.result) {
    throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
  }
}
