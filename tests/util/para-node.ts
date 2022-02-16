import tcpPortUsed from "tcp-port-used";
import path from "path";
import fs from "fs";
import child_process from "child_process";
import { killAll, run } from "polkadot-launch";
import {
  BINARY_PATH,
  OVERRIDE_RUNTIME_PATH,
  RELAY_BINARY_PATH,
  RELAY_CHAIN_NODE_NAMES,
} from "./constants";
const debug = require("debug")("test:para-node");

export async function findAvailablePorts(parachainCount: number = 1) {
  // 2 nodes per prachain, and as many relaychain nodes
  const relayCount = parachainCount + 1;
  const paraNodeCount = parachainCount * 2; // * 2;
  const nodeCount = relayCount + paraNodeCount;
  const portCount = nodeCount * 3;
  const availablePorts = await Promise.all(
    new Array(portCount).fill(0).map(async (_, index) => {
      let selectedPort = 0;
      let endingPort = 65535;
      const portDistance: number = Math.floor((endingPort - 1024) / portCount);
      let port = 1024 + index * portDistance + (process.pid % portDistance);
      while (!selectedPort && port < endingPort) {
        try {
          const inUse = await tcpPortUsed.check(port, "127.0.0.1");
          if (!inUse) {
            selectedPort = port;
          }
        } catch (e) {
          console.log("caught err ", e);
        }
        port++;
      }
      if (!selectedPort) {
        throw new Error(`No available port`);
      }
      return selectedPort;
    })
  );

  return new Array(nodeCount).fill(0).map((_, index) => ({
    p2pPort: availablePorts[index * 3 + 0],
    rpcPort: availablePorts[index * 3 + 1],
    wsPort: availablePorts[index * 3 + 2],
  }));
}

// Stores if the node has already started.
// It is used when a test file contains multiple describeDevMoonbeam. Those are
// executed within the same PID and so would generate a race condition if started
// at the same time.
let nodeStarted = false;

export type ParachainOptions = {
  chain: "moonbase-local" | "moonriver-local" | "moonbeam-local";
  relaychain?: "rococo-local" | "westend-local" | "kusama-local" | "polkadot-local";
  // specify the version of the binary using tag. Ex: "v0.18.1"
  // "local" uses target/release/moonbeam binary
  binary?: "local" | string;
  // specify the version of the runtime using tag. Ex: "runtime-1103"
  // "local" uses target/release/wbuild/<runtime>-runtime/<runtime>_runtime.compact.compressed.wasm
  runtime?: "local" | string;
  numberOfParachains?: number;
};

export interface ParachainPorts {
  parachainId: number;
  ports: NodePorts[];
}

export interface NodePorts {
  p2pPort: number;
  rpcPort: number;
  wsPort: number;
}

const RUNTIME_DIRECTORY = "runtimes";
const BINARY_DIRECTORY = "binaries";
const SPECS_DIRECTORY = "specs";

// Downloads the runtime and return the filepath
export async function getRuntimeWasm(
  runtimeName: "moonbase" | "moonriver" | "moonbeam",
  runtimeTag: string
): Promise<string> {
  const runtimePath = path.join(RUNTIME_DIRECTORY, `${runtimeName}-${runtimeTag}.wasm`);

  if (runtimeTag == "local") {
    const builtRuntimePath = path.join(
      OVERRIDE_RUNTIME_PATH || `../target/release/wbuild/${runtimeName}-runtime/`,
      `${runtimeName}_runtime.compact.compressed.wasm`
    );

    const code = fs.readFileSync(builtRuntimePath);
    fs.writeFileSync(runtimePath, `0x${code.toString("hex")}`);
  } else if (!fs.existsSync(runtimePath)) {
    console.log(`     Missing ${runtimePath} locally, downloading it...`);
    child_process.execSync(
      `mkdir -p ${path.dirname(runtimePath)} && ` +
        `wget https://github.com/PureStake/moonbeam/releases/` +
        `download/${runtimeTag}/${runtimeName}-${runtimeTag}.wasm ` +
        `-O ${runtimePath}.bin`
    );
    const code = fs.readFileSync(`${runtimePath}.bin`);
    fs.writeFileSync(runtimePath, `0x${code.toString("hex")}`);
    console.log(`${runtimePath} downloaded !`);
  }
  return runtimePath;
}

// Downloads the binary and return the filepath
export async function getMoonbeamReleaseBinary(binaryTag: string): Promise<string> {
  const binaryPath = path.join(BINARY_DIRECTORY, `moonbeam-${binaryTag}`);
  if (!fs.existsSync(binaryPath)) {
    console.log(`     Missing ${binaryPath} locally, downloading it...`);
    child_process.execSync(
      `mkdir -p ${path.dirname(binaryPath)} &&` +
        ` wget https://github.com/PureStake/moonbeam/releases/download/${binaryTag}/moonbeam` +
        ` -O ${binaryPath} &&` +
        ` chmod u+x ${binaryPath}`
    );
    console.log(`${binaryPath} downloaded !`);
  }
  return binaryPath;
}

export async function getMoonbeamDockerBinary(binaryTag: string): Promise<string> {
  const sha = child_process.execSync(`git rev-list -1 ${binaryTag}`);
  if (!sha) {
    console.error(`Invalid runtime tag ${binaryTag}`);
    return;
  }
  const sha8 = sha.slice(0, 8);

  const binaryPath = path.join(BINARY_DIRECTORY, `moonbeam-${sha8}`);
  if (!fs.existsSync(binaryPath)) {
    if (process.platform != "linux") {
      console.error(`docker binaries are only supported on linux.`);
      process.exit(1);
    }
    const dockerImage = `purestake/moonbeam:sha-${sha8}`;

    console.log(`     Missing ${binaryPath} locally, downloading it...`);
    child_process.execSync(`mkdir -p ${path.dirname(binaryPath)} && \
        docker create --name moonbeam-tmp ${dockerImage} && \
        docker cp moonbeam-tmp:/moonbeam/moonbeam ${binaryPath} && \
        docker rm moonbeam-tmp`);
    console.log(`${binaryPath} downloaded !`);
  }
  return binaryPath;
}

export async function getRawSpecsFromTag(
  runtimeName: "moonbase" | "moonriver" | "moonbeam",
  tag: string
) {
  const specPath = path.join(SPECS_DIRECTORY, `${runtimeName}-${tag}-raw-specs.json`);
  if (!fs.existsSync(specPath)) {
    const binaryPath = await getMoonbeamDockerBinary(tag);

    child_process.execSync(
      `mkdir -p ${path.dirname(specPath)} && ` +
        `${binaryPath} build-spec --chain moonbase-local --raw > ${specPath}`
    );
  }
  return specPath;
}

export async function generateRawSpecs(
  binaryPath: string,
  runtimeName: "moonbase-local" | "moonriver-local" | "moonbeam-local"
) {
  const specPath = path.join(SPECS_DIRECTORY, `${runtimeName}-raw-specs.json`);
  if (!fs.existsSync(specPath)) {
    child_process.execSync(
      `mkdir -p ${path.dirname(specPath)} && ` +
        `${binaryPath} build-spec --chain moonbase-local --raw > ${specPath}`
    );
  }
  return specPath;
}

// This will start a parachain node, only 1 at a time (check every 100ms).
// This will prevent race condition on the findAvailablePorts which uses the PID of the process
// Returns ports for the 3rd parachain node
export async function startParachainNodes(options: ParachainOptions): Promise<{
  relayPorts: NodePorts[];
  paraPorts: ParachainPorts[];
}> {
  while (nodeStarted) {
    // Wait 100ms to see if the node is free
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
  }
  const relaychain = options.relaychain || "rococo-local";
  // For now we only support one, two or three parachains
  const numberOfParachains = [1, 2, 3].includes(options.numberOfParachains)
    ? options.numberOfParachains
    : 1;
  const parachainArray = new Array(numberOfParachains).fill(0);
  nodeStarted = true;
  // Each node will have 3 ports. There are 2 nodes per parachain, and as many relaychain nodes.
  // So numberOfPorts =  3 * 2 * parachainCount
  const ports = await findAvailablePorts(numberOfParachains);

  //Build hrmpChannels, all connected to first parachain
  const hrmpChannels = [];
  new Array(numberOfParachains - 1).fill(0).forEach((_, i) => {
    hrmpChannels.push({
      sender: 1000,
      recipient: 1000 * (i + 2),
      maxCapacity: 8,
      maxMessageSize: 512,
    });
    hrmpChannels.push({
      sender: 1000 * (i + 2),
      recipient: 1000,
      maxCapacity: 8,
      maxMessageSize: 512,
    });
  });

  const chain = options.chain || "moonbase-local";
  const paraBinary =
    !options.binary || options.binary == "local"
      ? BINARY_PATH
      : await getMoonbeamReleaseBinary(options.binary);
  const specs =
    !options.runtime || options.runtime == "local"
      ? await generateRawSpecs(paraBinary, chain)
      : await getRawSpecsFromTag(chain.split("-")[0] as any, options.runtime);

  // Build launchConfig
  const launchConfig = {
    relaychain: {
      bin: RELAY_BINARY_PATH,
      chain: relaychain,
      nodes: new Array(numberOfParachains + 1).fill(0).map((_, i) => {
        return {
          name: RELAY_CHAIN_NODE_NAMES[i],
          port: ports[i].p2pPort,
          rpcPort: ports[i].rpcPort,
          wsPort: ports[i].wsPort,
        };
      }),
      genesis: {
        runtime: {
          runtime_genesis_config: {
            configuration: {
              config: {
                validation_upgrade_frequency: 1,
                validation_upgrade_delay: 30,
              },
            },
          },
        },
      },
    },
    parachains: parachainArray.map((_, i) => {
      return {
        bin: paraBinary,
        chain: specs,
        nodes: [
          {
            port: ports[i * 2 + numberOfParachains + 1].p2pPort,
            rpcPort: ports[i * 2 + numberOfParachains + 1].rpcPort,
            wsPort: ports[i * 2 + numberOfParachains + 1].wsPort,
            name: "alice",
            flags: [
              "--log=info,rpc=trace,evm=trace,ethereum=trace,author=trace",
              "--unsafe-rpc-external",
              "--execution=wasm",
              "--no-prometheus",
              "--no-telemetry",
              "--rpc-cors=all",
              "--",
              "--execution=wasm",
              "--no-mdns",
              "--no-prometheus",
              "--no-telemetry",
              "--no-private-ipv4",
            ],
          },
          {
            port: ports[i * 2 + numberOfParachains + 2].p2pPort,
            rpcPort: ports[i * 2 + numberOfParachains + 2].rpcPort,
            wsPort: ports[i * 2 + numberOfParachains + 2].wsPort,
            name: "bob",
            flags: [
              "--log=info,rpc=trace,evm=trace,ethereum=trace,author=trace",
              "--unsafe-rpc-external",
              "--execution=wasm",
              "--no-prometheus",
              "--no-telemetry",
              "--rpc-cors=all",
              "--",
              "--execution=wasm",
              "--no-mdns",
              "--no-prometheus",
              "--no-telemetry",
              "--no-private-ipv4",
            ],
          },
        ],
      };
    }),
    simpleParachains: [],
    hrmpChannels: hrmpChannels,
    finalization: true,
  };

  const onProcessExit = function () {
    killAll();
  };
  const onProcessInterrupt = function () {
    process.exit(2);
  };

  process.once("exit", onProcessExit);
  process.once("SIGINT", onProcessInterrupt);

  await run("", launchConfig);

  return {
    relayPorts: new Array(numberOfParachains + 1).fill(0).map((_, i) => {
      return {
        p2pPort: ports[i].p2pPort,
        rpcPort: ports[i].rpcPort,
        wsPort: ports[i].wsPort,
      };
    }),

    paraPorts: parachainArray.map((_, i) => {
      return {
        parachainId: 1000 * (i + 1),
        ports: [
          {
            p2pPort: ports[i * 2 + numberOfParachains + 1].p2pPort,
            rpcPort: ports[i * 2 + numberOfParachains + 1].rpcPort,
            wsPort: ports[i * 2 + numberOfParachains + 1].wsPort,
          },
          {
            p2pPort: ports[i * 2 + numberOfParachains + 2].p2pPort,
            rpcPort: ports[i * 2 + numberOfParachains + 2].rpcPort,
            wsPort: ports[i * 2 + numberOfParachains + 2].wsPort,
          },
        ],
      };
    }),
  };
}

export async function stopParachainNodes() {
  killAll();
  await new Promise((resolve) => {
    // TODO: improve, make killAll async https://github.com/paritytech/polkadot-launch/issues/139
    process.stdout.write("Waiting 5 seconds for processes to shut down...");
    setTimeout(resolve, 5000);
    nodeStarted = false;
    console.log(" done");
  });
}
