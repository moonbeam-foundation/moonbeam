import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import { u8aConcat } from "@polkadot/util";
import yargs from "yargs";
import { getApiFor, NETWORK_YARGS_OPTIONS } from "./utils/networks";
import chalk from "chalk";

const debug = require("debug")("main");

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    address: {
      type: "number",
      description: "Filter given address",
    },
  }).argv;

const getPrecompileStorageKey = (address: number) => {
  const indexKey = `0x${address.toString(16).padStart(40, "0")}`;
  return `0x${Buffer.from(
    u8aConcat(
      xxhashAsU8a("EVM", 128),
      xxhashAsU8a("AccountCodes", 128),
      blake2AsU8a(indexKey, 128),
      indexKey
    )
  ).toString("hex")}`;
};

const KNOWN_PRECOMPILES = [
  {
    index: 1,
    name: "ECRecover",
  },
  {
    index: 2,
    name: "Sha256",
  },
  {
    index: 3,
    name: "ShRipemd160a256",
  },
  {
    index: 4,
    name: "Identity",
  },
  {
    index: 5,
    name: "Modexp",
  },
  {
    index: 6,
    name: "Bn128Add",
  },
  {
    index: 7,
    name: "Bn128Mul",
  },
  {
    index: 8,
    name: "Bn128Pairing",
  },
  {
    index: 1024,
    name: "Sha3FIPS256",
  },
  {
    index: 1025,
    name: "Dispatch - generic",
  },
  {
    index: 1026,
    name: "ECRecoverPublicKey",
  },
  {
    index: 2048,
    name: "ParachainStakingWrapper",
  },
  {
    index: 2049,
    name: "CrowdloanRewardsWrapper",
  },
];

const main = async () => {
  const api = await getApiFor(argv);

  const addresses = argv.address ? [argv.address] : KNOWN_PRECOMPILES.map((p) => p.index);

  for (const address of addresses) {
    const name = KNOWN_PRECOMPILES.find((p) => p.index == address)?.name || "";
    const storageKey = getPrecompileStorageKey(address);
    const code = (await api.rpc.state.getStorage(storageKey)) as any;
    const hasCode = !!code.toHuman();
    const color = hasCode ? chalk.green : chalk.red;
    console.log(
      `${color(
        `${(name ? `(${name}) ` : "").padStart(26)}${address.toString().padEnd(5)}`
      )} [${storageKey}]: ${hasCode ? code.toHex() : "None"}`
    );
  }
  api.disconnect();
};

main();
