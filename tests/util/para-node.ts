import child_process from "child_process";
import fs from "fs";
import path from "path";
import { killAll, run } from "polkadot-launch";
import tcpPortUsed from "tcp-port-used";
import {
  generateRawSpecs,
  getMoonbeamReleaseBinary,
  getPolkadotReleaseBinary,
  getRawSpecsFromTag,
} from "./binaries";

import {
  BINARY_PATH,
  DISPLAY_LOG,
  RELAY_BINARY_PATH,
  RELAY_CHAIN_NODE_NAMES,
  RELAY_LOG,
} from "./constants";

const debug = require("debug")("test:para-node");

const PORT_PREFIX = process.env.PORT_PREFIX && parseInt(process.env.PORT_PREFIX);
const NODE_KEYS: { key: string; id: string }[] = [
  {
    key: "0x0000000000000000000000000000000000000000000000000000000000000000",
    id: "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
  },
  {
    key: "0x1111111111111111111111111111111111111111111111111111111111111111",
    id: "12D3KooWPqT2nMDSiXUSx5D7fasaxhxKigVhcqfkKqrLghCq9jxz",
  },
  {
    key: "0x2222222222222222222222222222222222222222222222222222222222222222",
    id: "12D3KooWLdJAwPtyQ5RFnr9wGXsQzpf3P2SeqFbYkqbfVehLu4Ns",
  },
  {
    key: "0x3333333333333333333333333333333333333333333333333333333333333333",
    id: "12D3KooWBRFW3HkJCLKSWb4yG6iWRBpgNjbM4FFvNsL5T5JKTqrd",
  },
  {
    key: "0x4444444444444444444444444444444444444444444444444444444444444444",
    id: "12D3KooWQJzxKtEUvbt9BZ1uJyAMw2WSEQSShp4my4c3iikhW8Cf",
  },
  {
    key: "0x5555555555555555555555555555555555555555555555555555555555555555",
    id: "12D3KooWPBFzpNemfrwjMSTSENKAC6cDHxE2RXojcMJRwMtitDms",
  },
];

export async function findAvailablePorts(parachainCount: number = 1) {
  // 2 nodes per prachain, and as many relaychain nodes
  const relayCount = parachainCount + 1;
  const paraNodeCount = parachainCount * 2; // 2 nodes each;
  const paraEmbeddedNodeCount = paraNodeCount; // 2 nodes each;
  const nodeCount = relayCount + paraNodeCount + paraEmbeddedNodeCount;
  const portCount = nodeCount * 3;

  if (PORT_PREFIX) {
    return [
      ...new Array(relayCount).fill(0).map((_, index) => ({
        p2pPort: PORT_PREFIX * 1000 + 10 * index,
        rpcPort: PORT_PREFIX * 1000 + 10 * index + 1,
      })),
      ...new Array(paraNodeCount + paraEmbeddedNodeCount).fill(0).map((_, index) => ({
        p2pPort: PORT_PREFIX * 1000 + 100 + 10 * index,
        rpcPort: PORT_PREFIX * 1000 + 100 + 10 * index + 1,
      })),
    ];
  }
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
  }));
}

// Stores if the node has already started.
// It is used when a test file contains multiple describeDevMoonbeam. Those are
// executed within the same PID and so would generate a race condition if started
// at the same time.
let nodeStarted = false;

export type ParaRuntimeOpt = {
  chain: "moonbase-local" | "moonriver-local" | "moonbeam-local";
  // specify the version of the runtime using tag. Ex: "runtime-1103"
  // "local" uses
  // target/release/wbuild/<runtime>-runtime/<runtime>_runtime.compact.compressed.wasm
  runtime?: "local" | string;
};

export type ParaSpecOpt = {
  // specify the file to use to start the chain
  spec: string;
};

export type ParaTestOptions = {
  parachain: (ParaRuntimeOpt | ParaSpecOpt) & {
    // specify the version of the binary using tag. Ex: "v0.18.1"
    // "local" uses target/release/moonbeam binary
    binary?: "local" | string;
    basePath?: string;
  };
  paraId?: number;
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
}

// log listeners to kill at the end;
const logListener: child_process.ChildProcessWithoutNullStreams[] = [];

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

  // For simplicity, forces the first parachain node to run on default ports
  ports[numberOfParachains + 1].rpcPort = 9944;

  //Build hrmpChannels, all connected to first parachain
  const hrmpChannels: {
    sender: number;
    recipient: number;
    maxCapacity: number;
    maxMessageSize: number;
  }[] = [];
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

  const paraBinary =
    !options.parachain.binary || options.parachain.binary == "local"
      ? BINARY_PATH
      : await getMoonbeamReleaseBinary(options.parachain.binary);
  const paraSpecs =
    "spec" in options.parachain
      ? options.parachain.spec
      : !("runtime" in options.parachain) || options.parachain.runtime == "local"
      ? await generateRawSpecs(paraBinary, options.parachain.chain || "moonbase-local")
      : await getRawSpecsFromTag(
          options.parachain.chain || "moonbase-local",
          options.parachain.runtime
        );

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
              validation_upgrade_delay: 30,
              validation_upgrade_cooldown: 30,
            },
          },
        },
      },
    },
    "v0.9.16": {
      runtime: {
        runtime_genesis_config: {
          configuration: {
            config: {
              validation_upgrade_delay: 30,
              validation_upgrade_cooldown: 30,
            },
          },
        },
      },
    },
  };
  const genesis = (RELAY_GENESIS_PER_VERSION as any)[options?.relaychain?.binary] || {};
  // Build launchConfig
  const launchConfig = {
    relaychain: {
      bin: relayBinary,
      chain: relayChain,
      nodes: new Array(numberOfParachains + 1).fill(0).map((_, i) => {
        return {
          nodeKey: NODE_KEYS[2 + i].key,
          name: RELAY_CHAIN_NODE_NAMES[i],
          port: ports[i].p2pPort,
          rpcPort: ports[i].rpcPort,
          flags: [
            process.env.FORCE_COMPILED_WASM
              ? `--wasm-execution=compiled`
              : `--wasm-execution=interpreted-i-know-what-i-do`,
            RELAY_LOG
              ? `--log=${RELAY_LOG}`
              : "--log=parachain::candidate-backing=trace,parachain::candidate-selection=trace," +
                "parachain::pvf=trace,parachain::collator-protocol=trace," +
                "parachain::provisioner=trace",
            "--state-pruning=archive",
          ],
        };
      }),
      genesis,
    },
    parachains: parachainArray.map((_, i) => {
      return {
        id: options.paraId || 1000,
        bin: paraBinary,
        chain: paraSpecs,
        nodes: [
          {
            port: ports[i * 4 + numberOfParachains + 1].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 1].rpcPort,
            nodeKey: NODE_KEYS[i * 2 + numberOfParachains + 1].key,
            name: "alice",
            flags: [
              "--log=info,evm=trace,ethereum=trace," +
                "pallet_parachain_staking=error," +
                "cumulus-consensus=trace,cumulus-collator=trace," +
                "parachain::collator_protocol=trace,parachain::candidate-selection=trace," +
                "parachain::collation_generation=trace,parachain::filtering=trace",
              "--state-pruning=archive",
              "--unsafe-rpc-external",
              "--execution=wasm",
              "--no-hardware-benchmarks",
              process.env.FORCE_COMPILED_WASM
                ? `--wasm-execution=compiled`
                : `--wasm-execution=interpreted-i-know-what-i-do`,
              "--no-prometheus",
              "--no-telemetry",
              "--rpc-cors=all",
              "--",
              "--state-pruning=archive",
              "--execution=wasm",
              "--no-hardware-benchmarks",
              process.env.FORCE_COMPILED_WASM
                ? `--wasm-execution=compiled`
                : `--wasm-execution=interpreted-i-know-what-i-do`,
              "--no-mdns",
              "--no-prometheus",
              "--no-telemetry",
              `--port=${ports[i * 4 + numberOfParachains + 2].p2pPort}`,
              `--rpc-port=${ports[i * 4 + numberOfParachains + 2].rpcPort}`,
            ],
          },
          {
            port: ports[i * 4 + numberOfParachains + 3].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 3].rpcPort,
            nodeKey: NODE_KEYS[i * 2 + numberOfParachains + 3].key,
            name: "bob",
            flags: [
              "--log=info,evm=trace,ethereum=trace," +
                "pallet_parachain_staking=error," +
                "cumulus-consensus=trace,cumulus-collator=trace," +
                "parachain::collator_protocol=trace,parachain::candidate-selection=trace," +
                "parachain::collation_generation=trace,parachain::filtering=trace",
              "--unsafe-rpc-external",
              "--execution=wasm",
              "--state-pruning=archive",
              process.env.FORCE_COMPILED_WASM
                ? `--wasm-execution=compiled`
                : `--wasm-execution=interpreted-i-know-what-i-do`,
              "--no-hardware-benchmarks",
              "--no-prometheus",
              "--no-telemetry",
              "--rpc-cors=all",
              "--",
              "--execution=wasm",
              "--state-pruning=archive",
              process.env.FORCE_COMPILED_WASM
                ? `--wasm-execution=compiled`
                : `--wasm-execution=interpreted-i-know-what-i-do`,
              "--no-hardware-benchmarks",
              "--no-mdns",
              "--no-prometheus",
              "--no-telemetry",
              `--port=${ports[i * 4 + numberOfParachains + 4].p2pPort}`,
              `--rpc-port=${ports[i * 4 + numberOfParachains + 4].rpcPort}`,
            ],
          },
        ].filter((_, i) => !process.env.SINGLE_PARACHAIN_NODE || i < 1),
      };
    }),
    simpleParachains: [] as any[],
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
    logListener.push(tailProcess);
  };
  const runPromise = run("", launchConfig);
  if (DISPLAY_LOG) {
    new Array(numberOfParachains + 1).fill(0).forEach(async (_, i) => {
      listenTo(`${RELAY_CHAIN_NODE_NAMES[i]}.log`, `relay-${i}`);
    });
    parachainArray.forEach(async (_, i) => {
      const filenameNode1 = `${ports[i * 4 + numberOfParachains + 1].rpcPort}.log`;
      listenTo(filenameNode1, `para-${i}-0`);
      if (!process.env.SINGLE_PARACHAIN_NODE) {
        const filenameNode2 = `${ports[i * 4 + numberOfParachains + 3].rpcPort}.log`;
        listenTo(filenameNode2, `para-${i}-1`);
      }
    });
  }

  let raceTimer;
  await Promise.race([
    runPromise,
    new Promise(
      (_, reject) =>
        (raceTimer = setTimeout(
          () => reject(new Error("timeout")),
          "spec" in options.parachain ? 12000000 : 60000
        ))
    ),
  ]);
  clearTimeout(raceTimer);

  return {
    relayPorts: new Array(numberOfParachains + 1).fill(0).map((_, i) => {
      return {
        p2pPort: ports[i].p2pPort,
        rpcPort: ports[i].rpcPort,
      };
    }),

    paraPorts: parachainArray.map((_, i) => {
      return {
        parachainId: 1000 * (i + 1),
        ports: [
          {
            p2pPort: ports[i * 4 + numberOfParachains + 1].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 1].rpcPort,
          },
          {
            p2pPort: ports[i * 4 + numberOfParachains + 3].p2pPort,
            rpcPort: ports[i * 4 + numberOfParachains + 3].rpcPort,
          },
        ].filter((_, i) => !process.env.SINGLE_PARACHAIN_NODE || i < 1),
      };
    }),
  };
}

export async function stopParachainNodes() {
  killAll();
  logListener.forEach((process) => {
    process.kill();
  });
  await new Promise((resolve) => {
    // TODO: improve, make killAll async https://github.com/paritytech/polkadot-launch/issues/139
    process.stdout.write("Waiting 5 seconds for processes to shut down...");
    setTimeout(resolve, 5000);
    nodeStarted = false;
    console.log(" done");
  });
}
