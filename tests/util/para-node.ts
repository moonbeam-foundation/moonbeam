import tcpPortUsed from "tcp-port-used";
import path from "path";
import { killAll, run } from "polkadot-launch";
import { BINARY_PATH, RELAY_BINARY_PATH, DISPLAY_LOG, SPAWNING_TIME } from "./constants";
const debug = require("debug")("test:para-node");

export async function findAvailablePorts(extraParachain: boolean = false) {
  const numberOfNodes = extraParachain ? 8 : 4; // two nodes per parachain, as many relaychain nodes
  const numberOfPorts = numberOfNodes * 3;
  const availablePorts = await Promise.all(
    new Array(numberOfPorts).fill(0).map(async (_, index) => {
      let selectedPort = 0;
      let port = 1024 + index * 2000 + (process.pid % 2000);
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

  return new Array(numberOfNodes).fill(0).map((_, index) => ({
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
  chain:
    | "moonbase-local"
    | "moonriver-local"
    | "moonbeam-local"
    | "moonbase"
    | "moonriver"
    | "moonbeam";
  relaychain?: "rococo-local" | "westend-local" | "kusama-local" | "polkadot-local";
  numberOfParachains?: number;
};

export interface NodePorts {
  p2pPort: number;
  rpcPort: number;
  wsPort: number;
}

// This will start a parachain node, only 1 at a time (check every 100ms).
// This will prevent race condition on the findAvailablePorts which uses the PID of the process
// Returns ports for the 3rd parachain node
export async function startParachainNodes(options: ParachainOptions): Promise<{
  relayPorts: NodePorts[];
  paraPorts: NodePorts[][];
}> {
  while (nodeStarted) {
    // Wait 100ms to see if the node is free
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
  }
  const relaychain = options.relaychain || "rococo-local";
  // For now we only support one or two parachains
  const numberOfParachains =
    (options.numberOfParachains < 3 &&
      options.numberOfParachains > 0 &&
      options.numberOfParachains) ||
    1;
  const parachainArray = new Array(numberOfParachains).fill(0);
  nodeStarted = true;
  // Each node will have 3 ports. There are 4 nodes total (2 relay, 2 collators) - so 12 ports
  // Plus 2 nodes if we need a second parachain
  const ports = await findAvailablePorts(numberOfParachains === 2);
  console.log("PORTS", ports);
  const launchConfig = {
    relaychain: {
      bin: RELAY_BINARY_PATH,
      chain: relaychain,
      nodes:
        numberOfParachains === 2
          ? [
              {
                name: "Alice",
                port: ports[0].p2pPort,
                rpcPort: ports[0].rpcPort,
                wsPort: ports[0].wsPort,
              },
              {
                name: "Bob",
                port: ports[1].p2pPort,
                rpcPort: ports[1].rpcPort,
                wsPort: ports[1].wsPort,
              },
              {
                name: "Charlie",
                port: ports[2].p2pPort,
                rpcPort: ports[2].rpcPort,
                wsPort: ports[2].wsPort,
              },
              {
                name: "Dave",
                port: ports[3].p2pPort,
                rpcPort: ports[3].rpcPort,
                wsPort: ports[3].wsPort,
              },
            ]
          : [
              {
                name: "Alice",
                port: ports[0].p2pPort,
                rpcPort: ports[0].rpcPort,
                wsPort: ports[0].wsPort,
              },
              {
                name: "Bob",
                port: ports[1].p2pPort,
                rpcPort: ports[1].rpcPort,
                wsPort: ports[1].wsPort,
              },
            ],
      genesis: {
        runtime: {
          runtime_genesis_config: {
            parachainsConfiguration: {
              config: {
                validation_upgrade_frequency: 1,
                validation_upgrade_delay: 1,
              },
            },
          },
        },
      },
    },
    parachains: parachainArray.map((_, i) => {
      return {
        bin: BINARY_PATH,
        id: 1000 * (i + 1),
        balance: "1000000000000000000000",
        chain: options.chain,
        nodes: [
          {
            port: ports[(i + numberOfParachains) * 2].p2pPort,
            rpcPort: ports[(i + numberOfParachains) * 2].rpcPort,
            wsPort: ports[(i + numberOfParachains) * 2].wsPort,
            name: "alice",
            flags: [
              "--log=info,rpc=trace,evm=trace,ethereum=trace",
              "--unsafe-rpc-external",
              "--rpc-cors=all",
              "--",
              "--execution=wasm",
            ],
          },
          {
            port: ports[(i + numberOfParachains) * 2 + 1].p2pPort,
            rpcPort: ports[(i + numberOfParachains) * 2 + 1].rpcPort,
            wsPort: ports[(i + numberOfParachains) * 2 + 1].wsPort,
            name: "bob",
            flags: [
              "--log=info,rpc=trace,evm=trace,ethereum=trace",
              "--unsafe-rpc-external",
              "--rpc-cors=all",
              "--",
              "--execution=wasm",
            ],
          },
        ],
      };
    }),
    simpleParachains: [],
    hrmpChannels: numberOfParachains === 2?[{
			"sender": 1000,
			"recipient": 2000,
			"maxCapacity": 8,
			"maxMessageSize": 512
		},{
			"sender": 2000,
			"recipient": 1000,
			"maxCapacity": 8,
			"maxMessageSize": 512
		}]:[],
    finalization: true,
  };

  const onProcessExit = function () {
    killAll;
  };
  const onProcessInterrupt = function () {
    process.exit(2);
  };

  process.once("exit", onProcessExit);
  process.once("SIGINT", onProcessInterrupt);

  await run(path.join(__dirname, "../"), launchConfig);

  return {
    relayPorts:
      numberOfParachains === 2
        ? [
            {
              p2pPort: ports[0].p2pPort,
              rpcPort: ports[0].rpcPort,
              wsPort: ports[0].wsPort,
            },
            {
              p2pPort: ports[1].p2pPort,
              rpcPort: ports[1].rpcPort,
              wsPort: ports[1].wsPort,
            },
            {
              p2pPort: ports[2].p2pPort,
              rpcPort: ports[2].rpcPort,
              wsPort: ports[2].wsPort,
            },
            {
              p2pPort: ports[3].p2pPort,
              rpcPort: ports[3].rpcPort,
              wsPort: ports[3].wsPort,
            },
          ]
        : [
            {
              p2pPort: ports[0].p2pPort,
              rpcPort: ports[0].rpcPort,
              wsPort: ports[0].wsPort,
            },
            {
              p2pPort: ports[1].p2pPort,
              rpcPort: ports[1].rpcPort,
              wsPort: ports[1].wsPort,
            },
          ],
    paraPorts: parachainArray.map((_, i) => {
      return [
        {
          p2pPort: ports[(i + numberOfParachains) * 2].p2pPort,
          rpcPort: ports[(i + numberOfParachains) * 2].rpcPort,
          wsPort: ports[(i + numberOfParachains) * 2].wsPort,
        },
        {
          p2pPort: ports[(i + numberOfParachains) * 2 + 1].p2pPort,
          rpcPort: ports[(i + numberOfParachains) * 2 + 1].rpcPort,
          wsPort: ports[(i + numberOfParachains) * 2 + 1].wsPort,
        },
      ];
    }),
  };
}

export async function stopParachainNodes() {
  killAll();
  await new Promise((resolve) => {
    // TODO: improve, make killAll async https://github.com/paritytech/polkadot-launch/issues/139
    console.log("Waiting 10 seconds for processes to shut down...");
    setTimeout(resolve, 10000);
    console.log("... done");
  });
}
