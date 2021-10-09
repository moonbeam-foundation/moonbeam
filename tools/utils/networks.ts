import { WsProvider } from "@polkadot/api";
import chalk from "chalk";
import { ApiPromise } from "@polkadot/api";
import { typesBundle } from "../../moonbeam-types-bundle/dist";
import { listenBlocks, printBlockDetails, RealtimeBlockDetails } from "./monitoring";
import { Options } from "yargs";
import Web3 from "web3";

export type NETWORK_NAME = "stagenet" | "alphanet" | "moonsama" | "moonsilver" | "moonriver";

export const NETWORK_WS_URLS: { [name in NETWORK_NAME]: string } = {
  stagenet: "wss://wss.stagenet.moonbeam.gcp.purestake.run",
  alphanet: "wss://wss.testnet.moonbeam.network",
  moonsama: "wss://wss.moonsama.gcp.purestake.run",
  moonsilver: "wss://wss.moonsilver.moonbeam.network",
  moonriver: "wss://wss.moonriver.moonbeam.network",
};
export const NETWORK_HTTP_URLS: { [name in NETWORK_NAME]: string } = {
  stagenet: "https://rpc.stagenet.moonbeam.gcp.purestake.run",
  alphanet: "https://rpc.testnet.moonbeam.network",
  moonsama: "https://rpc.moonsama.gcp.purestake.run",
  moonsilver: "https://rpc.moonsilver.moonbeam.network",
  moonriver: "https://rpc.moonriver.moonbeam.network",
};
export const NETWORK_NAMES = Object.keys(NETWORK_WS_URLS) as NETWORK_NAME[];

export const NETWORK_CHAIN_MAPPING: { [name: string]: NETWORK_NAME } = {
  "Moonbase Stage": "stagenet",
  "Moonbase Alpha": "alphanet",
  Moonsama: "moonsama",
  Moonsilver: "moonsilver",
  Moonriver: "moonriver",
};

export const NETWORK_COLORS: { [name in NETWORK_NAME]: chalk.ChalkFunction } = {
  stagenet: chalk.blueBright,
  alphanet: chalk.greenBright,
  moonsama: chalk.magentaBright,
  moonsilver: chalk.yellowBright,
  moonriver: chalk.redBright,
};

export type NetworkOptions = {
  url: Options & { type: "string" };
  network: Options & { type: "string" };
  finalized: Options & { type: "boolean" };
};

type Argv = {
  url?: string;
  network?: string;
  finalized?: boolean;
};

export const NETWORK_YARGS_OPTIONS: NetworkOptions = {
  url: {
    type: "string",
    description: "Websocket url",
    conflicts: ["network"],
    string: true,
  },
  network: {
    type: "string",
    choices: NETWORK_NAMES,
    description: "Known network",
    string: true,
  },
  finalized: {
    type: "boolean",
    default: false,
    description: "listen to finalized only",
  },
};

export function isKnownNetwork(name: string): name is NETWORK_NAME {
  return NETWORK_NAMES.includes(name as NETWORK_NAME);
}

export const getWsProviderForNetwork = (name: NETWORK_NAME) => {
  return new WsProvider(NETWORK_WS_URLS[name]);
};

// Supports providing an URL or a known network
export const getWsProviderFor = (argv: Argv) => {
  if (isKnownNetwork(argv.network)) {
    return getWsProviderForNetwork(argv.network);
  }
  return new WsProvider(argv.url);
};

export const getApiFor = async (argv: Argv) => {
  const wsProvider = getWsProviderFor(argv);
  return await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle,
  });
};

export const getMonitoredApiFor = async (argv: Argv) => {
  const wsProvider = getWsProviderFor(argv);
  const api = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle,
  });
  const networkName = argv.url
    ? NETWORK_CHAIN_MAPPING[(await api.rpc.system.chain()).toString()]
    : argv.network;

  let previousBlockDetails: RealtimeBlockDetails = null;
  listenBlocks(api, argv.finalized, async (blockDetails) => {
    printBlockDetails(
      blockDetails,
      {
        prefix: isKnownNetwork(networkName)
          ? NETWORK_COLORS[networkName](networkName.padStart(10, " "))
          : undefined,
      },
      previousBlockDetails
    );
    previousBlockDetails = blockDetails;
  });
  return api;
};

export const getWeb3For = async (argv) => {
  if (isKnownNetwork(argv.network)) {
    return new Web3(NETWORK_WS_URLS[argv.network]);
  }
  return new Web3(argv.url);
};
