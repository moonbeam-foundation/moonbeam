/**
 *  Script to launch 2 relay and 2 parachain nodes.
 *  It contains pre-registered versions to allow easy run using Docker.
 *
 *  ports can be given using --port-prefix xx (default 34) using the following rule:
 *  - relay 1 - p2p (p2p: XX000, rpcPort: XX001, wsPort: XX002)
 *  - relay 2 - p2p (p2p: XX010, rpcPort: XX011, wsPort: XX012)
 *  - para 1 - p2p (p2p: XX100, rpcPort: XX101, wsPort: XX102)
 *  - para 2 - p2p (p2p: XX110, rpcPort: XX111, wsPort: XX112)
 *
 */

import yargs from "yargs";
import * as fs from "fs";
import * as path from "path";
import * as child_process from "child_process";
import { killAll, run } from "polkadot-launch";

// Description of the network to launch
type NetworkConfig = {
  // From which docker to take the binary
  docker?: string;
  // To use instead of docker to run local binary
  binary?: string;
  // What chain to run
  chain: string;
};

// Description of the parachain network
type ParachainConfig = NetworkConfig & {
  // Which relay (name) config to use
  relay: string;
};

const parachains: { [name: string]: ParachainConfig } = {
  "moonriver-genesis": {
    relay: "kusama-9040",
    chain: "moonriver-local",
    docker: "purestake/moonbeam:moonriver-genesis",
  },
  "moonriver-genesis-fast": {
    relay: "rococo-9004",
    chain: "moonriver-local",
    docker: "purestake/moonbeam:sha-153c4c4a",
  },
  "moonbase-0.8.2": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.8.2",
  },
  "moonbase-0.8.1": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.8.1",
  },
  "moonbase-0.8.0": {
    relay: "rococo-9001",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.8.0",
  },
  "moonbase-0.9.2": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.9.2",
  },
  "moonbase-0.9.4": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.9.4",
  },
  "moonbase-0.9.6": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.9.6",
  },
  "moonbase-0.10.0": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.10.0",
  },
  "moonbase-0.11.3": {
    relay: "rococo-9004",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.11.3",
  },
  "moonbase-0.12.3": {
    relay: "rococo-9102",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.12.3",
  },
  "moonbase-0.13.2": {
    relay: "rococo-9100",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.13.2",
  },
  "moonbase-0.14.2": {
    relay: "rococo-9111",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.14.2",
  },
  "moonbase-0.15.1": {
    relay: "rococo-9111",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.15.1",
  },
  "moonbase-0.16.0": {
    relay: "rococo-9130",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.16.0",
  },
  "moonbase-0.17.0": {
    relay: "rococo-9130",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.17.0",
  },
  "moonbase-0.18.1": {
    relay: "rococo-9130",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.18.1",
  },
  "moonbase-0.19.2": {
    relay: "rococo-9130",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.19.2",
  },
  "moonbase-0.20.1": {
    relay: "rococo-9140",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.20.1",
  },
  "moonbase-0.21.0": {
    relay: "rococo-9140",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.21.0",
  },
  "moonbase-0.22.0": {
    relay: "rococo-9180",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.22.0",
  },
  "moonbase-0.23.0": {
    relay: "rococo-9180",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.23.0",
  },
  "moonbase-0.24.0": {
    relay: "rococo-9180",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.24.0",
  },
  "moonbase-0.25.0": {
    relay: "rococo-9230",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.25.0",
  },
  "moonbase-0.26.0": {
    relay: "rococo-9230",
    chain: "moonbase-local",
    docker: "purestake/moonbeam:v0.26.0",
  },
  local: {
    relay: "rococo-9230",
    chain: "moonbase-local",
    binary: "../target/release/moonbeam",
  },
};
const parachainNames = Object.keys(parachains);

const relays: { [name: string]: NetworkConfig } = {
  "kusama-9030": {
    docker: "purestake/moonbase-relay-testnet:sha-aa386760",
    chain: "kusama-local",
  },
  "kusama-9040": {
    docker: "purestake/moonbase-relay-testnet:sha-2f28561a",
    chain: "kusama-local",
  },
  "kusama-9030-fast": {
    docker: "purestake/moonbase-relay-testnet:sha-832cc0af",
    chain: "kusama-local",
  },
  "kusama-9040-fast": {
    docker: "purestake/moonbase-relay-testnet:sha-2239072e",
    chain: "kusama-local",
  },
  "rococo-9001": {
    docker: "purestake/moonbase-relay-testnet:sha-86a45114",
    chain: "rococo-local",
  },
  "rococo-9003": {
    docker: "purestake/moonbase-relay-testnet:sha-aa386760",
    chain: "rococo-local",
  },
  "rococo-9100": {
    docker: "purestake/moonbase-relay-testnet:v0.9.10",
    chain: "rococo-local",
  },
  "rococo-9102": {
    docker: "purestake/moonbase-relay-testnet:sha-43d9b899",
    chain: "rococo-local",
  },
  "rococo-9004": {
    docker: "purestake/moonbase-relay-testnet:sha-2f28561a",
    chain: "rococo-local",
  },
  "rococo-9111": {
    docker: "purestake/moonbase-relay-testnet:sha-7da182da",
    chain: "rococo-local",
  },
  "rococo-9130": {
    docker: "purestake/moonbase-relay-testnet:sha-45c0f1f3",
    chain: "rococo-local",
  },
  "rococo-9140": {
    docker: "purestake/moonbase-relay-testnet:sha-1a88d697",
    chain: "rococo-local",
  },
  "rococo-9180": {
    docker: "purestake/moonbase-relay-testnet:sha-f0dc95a6",
    chain: "rococo-local",
  },
  "rococo-9230": {
    docker: "purestake/moonbase-relay-testnet:sha-2fd38f09",
    chain: "rococo-local",
  },
  "westend-9030": {
    docker: "purestake/moonbase-relay-testnet:sha-aa386760",
    chain: "westend-local",
  },
  "westend-9040": {
    docker: "purestake/moonbase-relay-testnet:sha-2f28561a",
    chain: "westend-local",
  },
  local: {
    binary: "../../polkadot/target/release/polkadot",
    chain: "rococo-local",
  },
};
const relayNames = Object.keys(relays);

// We support 3 parachains for now
const validatorNames = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

const retrieveBinaryFromDocker = async (binaryPath: string, dockerImage: string) => {
  if (process.platform != "linux") {
    console.error(
      `docker binaries are only supported on linux. Use "local" config for compiled binaries`
    );
    process.exit(1);
  }
  const parachainPath = path.join(__dirname, binaryPath);
  if (!fs.existsSync(parachainPath)) {
    console.log(`     Missing ${binaryPath} locally, downloading it...`);
    child_process.execSync(`mkdir -p ${path.dirname(parachainPath)} && \
        docker create --name moonbeam-tmp ${dockerImage} && \
        docker cp moonbeam-tmp:/moonbeam/moonbeam ${parachainPath} && \
        docker rm moonbeam-tmp`);
    console.log(`${binaryPath} downloaded !`);
  }
};

async function start() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run launch [args]")
    .version("1.0.0")
    .options({
      parachain: {
        type: "string",
        choices: parachainNames,
        default: "local",
        describe: "which parachain configuration to run",
      },
      "parachain-chain": {
        type: "string",
        describe: "overrides parachain chain/runtime",
      },
      "parachain-runtime": {
        type: "string",
        describe: "<git-tag> to use for runtime specs",
        conflicts: ["parachain-chain"],
      },
      "parachain-id": { type: "number", default: 1000, describe: "overrides parachain-id" },
      relay: {
        type: "string",
        choices: relayNames,
        describe: "overrides relay configuration",
      },
      "relay-chain": {
        type: "string",
        choices: [
          "rococo",
          "westend",
          "kusama",
          "polkadot",
          "rococo-local",
          "westend-local",
          "kusama-local",
          "polkadot-local",
        ],
        describe: "overrides relay chain/runtime",
      },
      "port-prefix": {
        type: "number",
        default: 34,
        check: (port) => port >= 0 && port <= 64,
        describe: "provides port prefix for nodes",
      },
    })
    .help().argv;

  const portPrefix = argv["port-prefix"] || 34;
  const startingPort = portPrefix * 1000;
  let paras = [];
  let parasNames = [];
  let parachainsChains = [];
  let paraIds = [];

  // We start gathering all the information about the parachains
  if (Array.isArray(argv["parachain-id"])) {
    // We need two validators per parachain, so there is a maximum we can support
    if (argv["parachain-id"].length * 2 > validatorNames.length) {
      console.error(`Exceeded max number of paras: ${validatorNames.length / 2}`);
      return;
    }
    for (let i = 0; i < argv["parachain-id"].length; i++) {
      paraIds.push(argv["parachain-id"][i]);
    }
  }

  if (argv["parachain-runtime"]) {
    const sha = child_process.execSync(`git rev-list -1 ${argv["parachain-runtime"]}`);
    if (!sha) {
      console.error(`Invalid runtime tag ${argv["parachain-runtime"]}`);
      return;
    }
    const sha8 = sha.slice(0, 8);
    console.log(`Using runtime from sha: ${sha8}`);

    const parachainBinary = `build/sha-${sha8}/moonbeam`;
    const parachainPath = path.join(__dirname, parachainBinary);
    retrieveBinaryFromDocker(parachainBinary, `purestake/moonbeam:sha-${sha8}`);

    child_process.execSync(
      `${parachainBinary} build-spec --chain moonbase-local --raw > ` +
        `moonbase-${argv["parachain-runtime"]}-raw-spec.json`
    );
  }

  if (Array.isArray(argv.parachain)) {
    for (let i = 0; i < argv.parachain.length; i++) {
      if (i >= paraIds.length) {
        // If no paraId was provided for all of them, we just start assigning defaults
        // But if one of the defaults was assigned to a previous para, we error
        if (paraIds.includes(1000 + i)) {
          console.error(`Para id already included as default: ${1000 + i}`);
          return;
        } else {
          paraIds.push(1000 + i);
        }
      }
      const parachainName = argv.parachain[i].toString();
      parasNames.push(parachainName);
      paras.push(parachains[parachainName]);
      if (argv["parachain-runtime"]) {
        parachainsChains.push(`moonbase-${argv["parachain-runtime"]}-raw-spec.json`);
      }
      // If it is an array, push the position at which we are
      else if (Array.isArray(argv["parachain-chain"])) {
        parachainsChains.push(argv["parachain-chain"] || parachains[parachainName].chain);
      }
      // Else, push the value to the first parachain if it exists, else the default
      else {
        if (i == 0) {
          parachainsChains.push(argv["parachain-chain"] || parachains[parachainName].chain);
        } else {
          parachainsChains.push(parachains[parachainName].chain);
        }
      }
    }
  }
  // If it is not an array, we just simply push it
  else {
    paraIds.push(argv["parachain-id"] || 1000);
    const parachainName = argv.parachain.toString();
    parasNames.push(parachainName);
    paras.push(parachains[parachainName]);

    parachainsChains.push(
      argv["parachain-runtime"]
        ? `moonbase-${argv["parachain-runtime"]}-raw-spec.json`
        : argv["parachain-chain"] || parachains[parachainName].chain
    );
  }

  const relayName = argv.relay || paras[0].relay;

  if (!relayName || !relayNames.includes(relayName)) {
    console.error(`Invalid relay name: ${relayName}`);
    console.error(`Expected one of: ${relayNames.join(", ")}`);
    return;
  }

  const relay = relays[relayName];
  const relayChain = argv["relay-chain"] || relay.chain;

  console.log(
    `🚀 Relay:     ${relayName.padEnd(20)} - ${relay.docker || relay.binary} (${relayChain})`
  );

  let parachainBinaries = [];
  let parachainPaths = [];

  // We retrieve the binaries and paths for all parachains
  for (let i = 0; i < paras.length; i++) {
    if (paras[i].binary) {
      parachainBinaries.push(paras[i].binary);
      const parachainPath = path.join(__dirname, paras[i].binary);
      if (!fs.existsSync(parachainPath)) {
        console.log(`     Missing ${parachainPath}`);
        return;
      }
      parachainPaths.push(parachainPath);
    } else {
      const parachainBinary = `build/${parasNames[i]}/moonbeam`;
      const parachainPath = path.join(__dirname, parachainBinary);

      retrieveBinaryFromDocker(parachainBinary, paras[i].docker);
      parachainBinaries.push(parachainBinary);
      parachainPaths.push(parachainPath);
    }
    console.log(
      `🚀 Parachain: ${parasNames[i].padEnd(20)} - ${paras[i].docker || paras[i].binary} (${
        parachainsChains[i]
      })`
    );
  }

  let relayBinary;
  if (relay.binary) {
    relayBinary = relay.binary;
    const relayPath = path.join(__dirname, relay.binary);
    if (!fs.existsSync(relayPath)) {
      console.log(`     Missing ${relayPath}`);
      return;
    }
  } else {
    if (process.platform != "linux") {
      console.log(
        `docker binaries are only supported on linux. Use "local" config for compiled binaries`
      );
      return;
    }
    relayBinary = `build/${relayName}/polkadot`;
    const relayPath = path.join(__dirname, `build/${relayName}/polkadot`);
    if (!fs.existsSync(relayPath)) {
      console.log(`     Missing ${relayBinary} locally, downloading it...`);
      child_process.execSync(`mkdir -p ${path.dirname(relayPath)} && \
          docker create --name polkadot-tmp ${relay.docker} && \
          docker cp polkadot-tmp:/usr/local/bin/polkadot ${relayPath} && \
          docker rm polkadot-tmp`);
      console.log(`     ${relayBinary} downloaded !`);
    }
  }
  console.log("");

  let launchConfig = launchTemplate;
  launchConfig.relaychain.bin = relayBinary;
  launchConfig.relaychain.chain = relayChain;

  let relay_nodes = [];
  // We need to build the configuration for each of the paras
  for (let i = 0; i < parachainBinaries.length; i++) {
    let relayNodeConfig = JSON.parse(JSON.stringify(relayNodeTemplate));
    let parachainConfig = JSON.parse(JSON.stringify(parachainTemplate));
    // HRMP is not configurable in Kusama and Westend thorugh genesis. We should detect this here
    // Maybe there is a nicer way of doing this
    if (launchConfig.relaychain.chain.startsWith("rococo")) {
      // Create HRMP channels
      // HRMP channels are uni-directonal, we need to create both ways
      for (let j = 0; j < paraIds.length; j++) {
        let hrmpConfig = JSON.parse(JSON.stringify(hrmpTemplate));
        if (j != i) {
          hrmpConfig.sender = paraIds[i];
          hrmpConfig.recipient = paraIds[j];
          launchConfig.hrmpChannels.push(hrmpConfig);
        }
      }
    }

    parachainConfig.bin = parachainBinaries[i];
    parachainConfig.chain = parachainsChains[i];
    parachainConfig.id = paraIds[i];

    parachainConfig.nodes.forEach((node, index) => {
      node.port = startingPort + 100 + i * 100 + index * 10;
      node.rpcPort = startingPort + 101 + i * 100 + index * 10;
      node.wsPort = startingPort + 102 + i * 100 + index * 10;
    });

    launchConfig.parachains.push(parachainConfig);

    // Two relay nodes per para
    relayNodeConfig[0].name = validatorNames[i * 2];
    relayNodeConfig[0].port = startingPort + i * 20;
    relayNodeConfig[0].rpcPort = startingPort + i * 20 + 1;
    relayNodeConfig[0].wsPort = startingPort + i * 20 + 2;

    relayNodeConfig[1].name = validatorNames[i * 2 + 1];
    relayNodeConfig[1].port = startingPort + i * 20 + 10;
    relayNodeConfig[1].rpcPort = startingPort + i * 20 + 11;
    relayNodeConfig[1].wsPort = startingPort + i * 20 + 12;
    relay_nodes.push(relayNodeConfig[0]);
    relay_nodes.push(relayNodeConfig[1]);
  }

  launchConfig.relaychain.nodes = relay_nodes;

  const knownRelayChains = ["kusama", "westend", "rococo", "polkadot"]
    .map((network) => [`${network}`, `${network}-local`, `${network}-dev`])
    .flat();

  // In case the chain is a spec file
  if (!knownRelayChains.includes(launchConfig.relaychain.chain)) {
    delete launchConfig.relaychain.genesis;
  } else if (launchConfig.relaychain.chain.startsWith("rococo")) {
    // To support compatibility with rococo
    (launchConfig.relaychain.genesis.runtime as any).runtime_genesis_config = {
      ...launchConfig.relaychain.genesis.runtime,
    };
    for (let key of Object.keys(launchConfig.relaychain.genesis.runtime)) {
      if (key != "runtime_genesis_config") {
        delete launchConfig.relaychain.genesis.runtime[key];
      }
    }
  }

  // Kill all processes when exiting.
  process.on("exit", function () {
    killAll();
  });

  // Handle ctrl+c to trigger `exit`.
  process.on("SIGINT", function () {
    process.exit(2);
  });

  await run(__dirname, launchConfig);
}

const launchTemplate = {
  relaychain: {
    bin: "...",
    chain: "...",
    nodes: [],
    genesis: {
      runtime: {
        configuration: {
          config: {
            validation_upgrade_frequency: 1,
            validation_upgrade_delay: 30,
            validation_upgrade_cooldown: 30,
          },
        },
      },
    },
  },
  parachains: [],
  simpleParachains: [],
  hrmpChannels: [],
  types: {
    Address: "MultiAddress",
    LookupSource: "MultiAddress",
    RoundIndex: "u32",
  },
  finalization: true,
};

const relayNodeTemplate = [
  {
    name: "alice",
    flags: ["--log=info,parachain::pvf=trace"],
    port: 0,
    rpcPort: 1,
    wsPort: 2,
  },
  {
    name: "bob",
    flags: ["--log=info,parachain::pvf=trace"],
    port: 10,
    rpcPort: 11,
    wsPort: 12,
  },
];

const parachainTemplate = {
  bin: "...",
  id: 1000,
  balance: "1000000000000000000000",
  chain: "...",
  nodes: [
    {
      port: 100,
      rpcPort: 101,
      wsPort: 102,
      name: "alice",
      flags: [
        "--unsafe-rpc-external",
        "--unsafe-ws-external",
        "--rpc-methods=Unsafe",
        "--rpc-cors=all",
        "--",
        "--execution=wasm",
      ],
    },
    {
      port: 110,
      rpcPort: 111,
      wsPort: 112,
      name: "bob",
      flags: [
        "--unsafe-rpc-external",
        "--unsafe-ws-external",
        "--rpc-methods=Unsafe",
        "--rpc-cors=all",
        "--",
        "--execution=wasm",
      ],
    },
  ],
};

const hrmpTemplate = {
  sender: "200",
  recipient: "300",
  maxCapacity: 8,
  maxMessageSize: 32768,
};

start();
