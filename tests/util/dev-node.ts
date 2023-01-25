import { ChildProcess, spawn } from "child_process";
import tcpPortUsed from "tcp-port-used";
import { StringMappingType } from "typescript";

import {
  BASE_PATH,
  BINARY_PATH,
  CUSTOM_SPEC_PATH,
  DISPLAY_LOG,
  ETHAPI_CMD,
  MOONBEAM_LOG,
  SPAWNING_TIME,
  WASM_RUNTIME_OVERRIDES,
} from "./constants";

const debug = require("debug")("test:dev-node");

export async function findAvailablePorts() {
  const availablePorts = await Promise.all(
    [null, null, null].map(async (_, index) => {
      let selectedPort = 0;
      let port = 1024 + index * 20000 + (process.pid % 20000);
      let endingPort = 65535;
      while (!selectedPort && port < endingPort) {
        const inUse = await tcpPortUsed.check(port, "127.0.0.1");
        if (!inUse) {
          selectedPort = port;
        }
        port++;
      }
      if (!selectedPort) {
        throw new Error(`No available port`);
      }
      return selectedPort;
    })
  );

  return {
    p2pPort: availablePorts[0],
    rpcPort: availablePorts[1],
    wsPort: availablePorts[2],
  };
}

export type RuntimeChain = "moonbase" | "moonriver" | "moonbeam";

// Stores if the node has already started.
// It is used when a test file contains multiple describeDevMoonbeam. Those are
// executed within the same PID and so would generate a race condition if started
// at the same time.
let nodeStarted = false;

// This will start a moonbeam dev node, only 1 at a time (check every 100ms).
// This will prevent race condition on the findAvailablePorts which uses the PID of the process
export async function startMoonbeamDevNode(
  withWasm?: boolean,
  runtime: RuntimeChain = "moonbase"
): Promise<{
  p2pPort: number;
  rpcPort: number;
  wsPort: number;
  runningNode: ChildProcess;
}> {
  while (nodeStarted) {
    // Wait 100ms to see if the node is free
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
  }
  nodeStarted = true;
  const { p2pPort, rpcPort, wsPort } = await findAvailablePorts();

  if (process.env.FORCE_WASM_EXECUTION == "true") {
    withWasm = true;
  }

  const cmd = BINARY_PATH;
  let args = [
    withWasm ? `--execution=Wasm` : `--execution=Native`, // Faster execution using native
    process.env.FORCE_COMPILED_WASM
      ? `--wasm-execution=compiled`
      : `--wasm-execution=interpreted-i-know-what-i-do`,
    ETHAPI_CMD != "" ? `${ETHAPI_CMD}` : `--ethapi=txpool`,
    `--no-hardware-benchmarks`,
    `--no-telemetry`,
    `--reserved-only`,
    `--no-grandpa`,
    `--no-prometheus`,
    `--force-authoring`,
    `--rpc-cors=all`,
    `--alice`,
    `--chain=${runtime}-dev`,
    `--sealing=manual`,
    `--in-peers=0`,
    `--out-peers=0`,
    `-l${MOONBEAM_LOG}`,
    `--port=${p2pPort}`,
    `--rpc-port=${rpcPort}`,
    `--ws-port=${wsPort}`,
    `--tmp`,
  ];
  if (WASM_RUNTIME_OVERRIDES != "") {
    args.push(`--wasm-runtime-overrides=${WASM_RUNTIME_OVERRIDES}`);
    // For tracing tests now we require to enable archive block pruning.
    args.push(`--blocks-pruning=archive`);
  } else if (ETHAPI_CMD != "") {
    args.push("--wasm-runtime-overrides=/");
  }
  debug(`Starting dev node: --port=${p2pPort} --rpc-port=${rpcPort} --ws-port=${wsPort}`);

  const onProcessExit = function () {
    runningNode && runningNode.kill();
  };
  const onProcessInterrupt = function () {
    process.exit(2);
  };

  let runningNode: ChildProcess = null;
  process.once("exit", onProcessExit);
  process.once("SIGINT", onProcessInterrupt);
  runningNode = spawn(cmd, args);

  runningNode.once("exit", () => {
    process.removeListener("exit", onProcessExit);
    process.removeListener("SIGINT", onProcessInterrupt);
    nodeStarted = false;
    debug(`Exiting dev node: --port=${p2pPort} --rpc-port=${rpcPort} --ws-port=${wsPort}`);
  });

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

  const binaryLogs: any[] = [];
  await new Promise<void>((resolve) => {
    const timer = setTimeout(() => {
      console.error(`\x1b[31m Failed to start Moonbeam Test Node.\x1b[0m`);
      console.error(`Command: ${cmd} ${args.join(" ")}`);
      console.error(`Logs:`);
      console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
      throw new Error("Failed to launch node");
    }, SPAWNING_TIME - 2000);

    const onData = async (chunk: any) => {
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

  return { p2pPort, rpcPort, wsPort, runningNode };
}

// This will start a moonbeam dev node from forked state, that has been previously setup with
// a snapshot of production state via the moonbeam-tools run-fork-solo command
export async function startMoonbeamForkedNode(
  rpcPort: number,
  wsPort: number
): Promise<{
  rpcPort: number;
  wsPort: number;
  runningNode: ChildProcess;
}> {
  while (nodeStarted) {
    // Wait 100ms to see if the node is free
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
  }
  nodeStarted = true;

  const cmd = BINARY_PATH;
  let args = [
    `--execution=Native`,
    `--no-hardware-benchmarks`,
    `--no-telemetry`,
    `--database=paritydb`,
    `--no-prometheus`,
    `--alice`,
    `--chain=${CUSTOM_SPEC_PATH}`,
    `--sealing=manual`,
    `-l${MOONBEAM_LOG}`,
    `--rpc-port=${rpcPort}`,
    `--ws-port=${wsPort}`,
    `--trie-cache-size=0`,
    `--db-cache=5000`,
    `--collator`,
    `--base-path=${BASE_PATH}`,
  ];

  debug(`Starting dev node: --rpc-port=${rpcPort} --ws-port=${wsPort}`);

  const onProcessExit = function () {
    runningNode && runningNode.kill();
  };
  const onProcessInterrupt = function () {
    process.exit(2);
  };

  let runningNode: ChildProcess = null;
  process.once("exit", onProcessExit);
  process.once("SIGINT", onProcessInterrupt);
  runningNode = spawn(cmd, args);

  runningNode.once("exit", () => {
    process.removeListener("exit", onProcessExit);
    process.removeListener("SIGINT", onProcessInterrupt);
    nodeStarted = false;
    debug(`Exiting dev node: --rpc-port=${rpcPort} --ws-port=${wsPort}`);
  });

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

  const binaryLogs: any[] = [];
  await new Promise<void>((resolve) => {
    const timer = setTimeout(() => {
      console.error(`\x1b[31m Failed to start Moonbeam Test Node.\x1b[0m`);
      console.error(`Command: ${cmd} ${args.join(" ")}`);
      console.error(`Logs:`);
      console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
      throw new Error("Failed to launch node");
    }, SPAWNING_TIME - 2000);

    const onData = async (chunk: any) => {
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

  return { rpcPort, wsPort, runningNode };
}
