import anyTest, { TestInterface } from "ava";
import getPort from "get-port";
import { spawn, ChildProcess } from "child_process";
import { BINARY_PATH, DISPLAY_LOG, MOONBEAM_LOG, SPAWNING_TIME } from "./constants";

export async function findAvailablePorts() {
  const availablePorts = await Promise.all([null, null, null].map(async (_, index) => getPort()));

  return {
    p2pPort: availablePorts[0],
    rpcPort: availablePorts[1],
    wsPort: availablePorts[2],
  };
}

export async function startMoonbeamDevNode(): Promise<{
  p2pPort: number;
  rpcPort: number;
  wsPort: number;
  runningNode: ChildProcess;
}> {
  const { p2pPort, rpcPort, wsPort } = await findAvailablePorts();

  // console.log("Using ports", { p2pPort, rpcPort, wsPort });
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

  let runningNode: ChildProcess = null;
  process.once("exit", function () {
    runningNode && runningNode.kill();
  });
  process.once("SIGINT", function () {
    process.exit(2);
  });
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

  return { p2pPort, rpcPort, wsPort, runningNode };
}
