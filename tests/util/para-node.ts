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
  const paraNodeCount = parachainCount * 2; // 2 nodes each;
  const paraEmbeddedNodeCount = paraNodeCount; // 2 nodes each;
  const nodeCount = relayCount + paraNodeCount + paraEmbeddedNodeCount;
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

export type ParaTestOptions = {
  parachain: {
    chain: "moonbase-local" | "moonriver-local" | "moonbeam-local";
    // specify the version of the binary using tag. Ex: "v0.18.1"
    // "local" uses target/release/moonbeam binary
    binary?: "local" | string;
    // specify the version of the runtime using tag. Ex: "runtime-1103"
    // "local" uses
    // target/release/wbuild/<runtime>-runtime/<runtime>_runtime.compact.compressed.wasm
    runtime?: "local" | string;
  };
  relaychain?: {
    chain?: "rococo-local" | "westend-local" | "kusama-local" | "polkadot-local";
    // specify the version of the binary using tag. Ex: "v0.9.13"
    // "local" uses target/release/polkadot binary
    binary?: "local" | string;
  };
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
        `wget -q https://github.com/PureStake/moonbeam/releases/` +
        `download/${runtimeTag}/${runtimeName}-${runtimeTag}.wasm ` +
        `-O ${runtimePath}.bin`
    );
    const code = fs.readFileSync(`${runtimePath}.bin`);
    fs.writeFileSync(runtimePath, `0x${code.toString("hex")}`);
    console.log(`${runtimePath} downloaded !`);
  }
  return runtimePath;
}

export async function getGithubReleaseBinary(url: string, binaryPath: string): Promise<string> {
  if (!fs.existsSync(binaryPath)) {
    console.log(`     Missing ${binaryPath} locally, downloading it...`);
    child_process.execSync(
      `mkdir -p ${path.dirname(binaryPath)} &&` +
        ` wget -q ${url}` +
        ` -O ${binaryPath} &&` +
        ` chmod u+x ${binaryPath}`
    );
    console.log(`${binaryPath} downloaded !`);
  }
  return binaryPath;
}

// Downloads the binary and return the filepath
export async function getMoonbeamReleaseBinary(binaryTag: string): Promise<string> {
  const binaryPath = path.join(BINARY_DIRECTORY, `moonbeam-${binaryTag}`);
  return getGithubReleaseBinary(
    `https://github.com/PureStake/moonbeam/releases/download/${binaryTag}/moonbeam`,
    binaryPath
  );
}
export async function getPolkadotReleaseBinary(binaryTag: string): Promise<string> {
  const binaryPath = path.join(BINARY_DIRECTORY, `polkadot-${binaryTag}`);
  return getGithubReleaseBinary(
    `https://github.com/paritytech/polkadot/releases/download/${binaryTag}/polkadot`,
    binaryPath
  );
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
export async function startParachainNodes(options: ParaTestOptions): Promise<{
  relayPorts: NodePorts[];
  paraPorts: ParachainPorts[];
}> {
  while (nodeStarted) {
    // Wait 100ms to see if the node is free
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
  }
  // For now we only support one, two or three parachains
  const numberOfParachains = [1, 2, 3].includes(options.numberOfParachains)
    ? options.numberOfParachains
    : 1;
  const parachainArray = new Array(numberOfParachains).fill(0);
  nodeStarted = true;
  // Each node will have 3 ports.
  // 2 parachains nodes per parachain.
  // 2 ports set (para + relay) per parachain node.
  // n+1 relay node.
  // So numberOfPorts =  3 * 2 * 2 * parachainCount
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

  const paraChain = options.parachain.chain || "moonbase-local";
  const paraBinary =
    !options.parachain.binary || options.parachain.binary == "local"
      ? BINARY_PATH
      : await getMoonbeamReleaseBinary(options.parachain.binary);
  const paraSpecs =
    !options.parachain.runtime || options.parachain.runtime == "local"
      ? await generateRawSpecs(paraBinary, paraChain)
      : await getRawSpecsFromTag(paraChain.split("-")[0] as any, options.parachain.runtime);

  const relayChain = options.relaychain?.chain || "rococo-local";
  const relayBinary =
    !options?.relaychain?.binary || options?.relaychain?.binary == "local"
      ? RELAY_BINARY_PATH
      : await getPolkadotReleaseBinary(options.relaychain.binary);

  const RELAY_GENESIS_PER_VERSION = {
    "v0.9.13": {
      runtime: {
        runtime_genesis_config: {
          configuration: {
            config: {
              validation_upgrade_frequency: 2,
              validation_upgrade_delay: 2,
            },
          },
        },
      },
    },
  };
  const genesis = RELAY_GENESIS_PER_VERSION[options?.relaychain?.binary] || {};
  // Build launchConfig
  const launchConfig = {
    relaychain: {
      bin: relayBinary,
      chain: relayChain,
      nodes: new Array(numberOfParachains + 1).fill(0).map((_, i) => {
        return {
          name: RELAY_CHAIN_NODE_NAMES[i],
          port: ports[i].p2pPort,
          rpcPort: ports[i].rpcPort,
          wsPort: ports[i].wsPort,
        };
      }),
      genesis,
    },
    parachains: parachainArray.map((_, i) => {
      return {
        bin: paraBinary,
        chain: paraSpecs,
        nodes: [
          {
            port: ports[i * 4 + numberOfParachains + 1].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 1].rpcPort,
            wsPort: ports[i * 4 + numberOfParachains + 1].wsPort,
            name: "alice",
            flags: [
              "--log=info,rpc=info,evm=trace,ethereum=trace,author=trace",
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
              `--port=${ports[i * 4 + numberOfParachains + 2].p2pPort}`,
              `--rpc-port=${ports[i * 4 + numberOfParachains + 2].rpcPort}`,
              `--ws-port=${ports[i * 4 + numberOfParachains + 2].wsPort}`,
            ],
          },
          {
            port: ports[i * 4 + numberOfParachains + 3].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 3].rpcPort,
            wsPort: ports[i * 4 + numberOfParachains + 3].wsPort,
            name: "bob",
            flags: [
              "--log=info,rpc=info,evm=trace,ethereum=trace,author=trace",
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
              `--port=${ports[i * 4 + numberOfParachains + 4].p2pPort}`,
              `--rpc-port=${ports[i * 4 + numberOfParachains + 4].rpcPort}`,
              `--ws-port=${ports[i * 4 + numberOfParachains + 4].wsPort}`,
            ],
          },
        ],
      };
    }),
    simpleParachains: [],
    hrmpChannels: hrmpChannels,
    finalization: true,
  };
  console.log(`Using`, JSON.stringify(launchConfig, null, 2));

  const onProcessExit = function () {
    killAll();
  };
  const onProcessInterrupt = function () {
    process.exit(2);
  };

  process.once("exit", onProcessExit);
  process.once("SIGINT", onProcessInterrupt);

  const listenTo = async (filename: string, prepend: string) => {
    while (!fs.existsSync(filename)) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    const tailProcess = child_process.spawn("tail", ["-f", filename]);
    tailProcess.stdout.on("data", function (data) {
      console.log(`${prepend} ${data.toString().trim()}`);
    });
  };
  const runPromise = run("", launchConfig);
  if (process.env.MOONBEAM_LOG) {
    new Array(numberOfParachains + 1).fill(0).forEach(async (_, i) => {
      listenTo(`${RELAY_CHAIN_NODE_NAMES[i]}.log`, `relay-${i}`);
    });
    parachainArray.forEach(async (_, i) => {
      const filenameNode1 = `${ports[i * 4 + numberOfParachains + 1].wsPort}.log`;
      listenTo(filenameNode1, `para-${i}-0`);
      const filenameNode2 = `${ports[i * 4 + numberOfParachains + 1].wsPort}.log`;
      listenTo(filenameNode2, `para-${i}-1`);
    });
  }

  await Promise.race([
    runPromise,
    new Promise((_, reject) => setTimeout(() => reject(new Error("timeout")), 60000)),
  ]);

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
            p2pPort: ports[i * 4 + numberOfParachains + 1].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 1].rpcPort,
            wsPort: ports[i * 4 + numberOfParachains + 1].wsPort,
          },
          {
            p2pPort: ports[i * 4 + numberOfParachains + 3].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 3].rpcPort,
            wsPort: ports[i * 4 + numberOfParachains + 3].wsPort,
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
