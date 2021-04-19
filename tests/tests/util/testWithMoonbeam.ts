import Web3 from "web3";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { typesBundle } from "../../../moonbeam-types-bundle";
import { HttpProvider } from "web3-core";
import { ethers } from "ethers";

import { spawn, ChildProcess, ChildProcessWithoutNullStreams } from "child_process";
import { BINARY_PATH, DISPLAY_LOG, MOONBEAM_LOG, SPAWNING_TIME, DEBUG_MODE } from "../constants";
import { ErrorReport } from "./fillBlockWithTx";
import { findAvailablePorts } from "./findAvailablePorts";

import { contractSources } from "../../contracts/sources";
import { getCompiled } from "./contracts";
export async function mochaGlobalSetup() {
  // First compile all contracts
  console.log("Making sure all contracts are compiled...");
  await Promise.all(
    Object.keys(contractSources).map(async (contractName) => {
      return getCompiled(contractName);
    })
  );
}

export function log(...msg: (string | number | ErrorReport)[]) {
  if (process.argv && process.argv[2] && process.argv[2] === "--printlogs") {
    console.log(...msg);
  }
}

export interface Context {
  web3: Web3;
  ethers: ethers.providers.JsonRpcProvider;

  // WsProvider for the PolkadotJs API
  wsProvider: WsProvider;
  polkadotApi: ApiPromise;
}

let runningNode: ChildProcessWithoutNullStreams;

export async function startMoonbeamNode(
  //TODO Make this parameter optional and just default to development.
  // For now I'm just ignoring the param and hardcoding development below.
  specFilename: string,
  provider?: string
): Promise<{ context: Context; runningNode: ChildProcess }> {
  let { p2pPort, rpcPort, wsPort } = await findAvailablePorts();

  const cmd = BINARY_PATH;
  const args = [
    `--execution=Native`, // Faster execution using native
    `--no-telemetry`,
    `--no-prometheus`,
    `--dev`,
    `--ethapi=txpool,debug,trace`,
    `--sealing=manual`,
    `-l${MOONBEAM_LOG}`,
    `--port=${p2pPort}`,
    `--rpc-port=${rpcPort}`,
    `--ws-port=${wsPort}`,
    `--tmp`,
  ];

  if (!DEBUG_MODE) {
    runningNode = spawn(cmd, args);
    runningNode.on("error", (err) => {
      if ((err as any).errno == "ENOENT") {
        console.error(
          `\x1b[31mMissing Moonbeam binary ` +
            `(${BINARY_PATH}).\nPlease compile the Moonbeam project\x1b[0m`
        );
      } else {
        console.error(err);
      }
      process.exit(1);
    });

    const binaryLogs = [];
    await new Promise<void>((resolve) => {
      const timer = setTimeout(() => {
        console.error(`\x1b[31m Failed to start Moonbeam Test Node.\x1b[0m`);
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
        if (chunk.toString().match(/Development Service Ready/)) {
          clearTimeout(timer);
          if (!DISPLAY_LOG) {
            runningNode.stderr.off("data", onData);
            runningNode.stdout.off("data", onData);
          }
          resolve();
        }
      };
      runningNode.stderr.on("data", onData);
      runningNode.stdout.on("data", onData);
    });
  } else {
    wsPort = 19933;
    rpcPort = 19932;
  }

  const wsProvider = new WsProvider(`ws://localhost:${wsPort}`);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });

  let web3 =
    provider == "ws"
      ? new Web3(`ws://localhost:${wsPort}`)
      : new Web3(`http://localhost:${rpcPort}`);

  const ethersApi = new ethers.providers.JsonRpcProvider(
    (web3.currentProvider as HttpProvider).host
  );

  // This is needed as the EVM runtime needs to warmup with a first call
  await web3.eth.getChainId();

  // This is needed as it takes a bit more time than when the log is printed to have all the modules
  // initialized in the node. (getBalance would fail otherwise)
  await polkadotApi.rpc.chain.getBlock();

  return { context: { ethers: ethersApi, web3, polkadotApi, wsProvider }, runningNode };
}

process.once("exit", function () {
  runningNode && runningNode.kill();
});

// Handle ctrl+c to trigger `exit`.
process.once("SIGINT", function () {
  process.exit(2);
});

export function describeWithMoonbeam(
  title: string,
  specFilename: string,
  cb: (context: Context) => void,
  provider?: string
) {
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout(5000);

    let context: Context = { ethers: null, web3: null, wsProvider: null, polkadotApi: null };
    let binary: ChildProcess;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      this.timeout(SPAWNING_TIME);
      const init = await startMoonbeamNode(specFilename, provider);
      // Context is given prior to this assignement, so doing
      // context = init.context will fail because it replace the variable;
      context.web3 = init.context.web3;
      context.ethers = init.context.ethers;
      context.wsProvider = init.context.wsProvider;
      context.polkadotApi = init.context.polkadotApi;
      binary = init.runningNode;
    });

    after(async function () {
      // console.log(`\x1b[31m Killing RPC\x1b[0m`);
      context.wsProvider.disconnect();
      if (!DEBUG_MODE) {
        binary.kill();
      }
      binary = null;
    });

    cb(context);
  });
}
