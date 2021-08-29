import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { BlockHash, Extrinsic } from "@polkadot/types/interfaces";
import { xxhashAsU8a, blake2AsHex } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import yargs from "yargs";

const debug = require("debug")("main");

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    module: {
      type: "string",
      description: "name of the module (ex: ParachainStaking)",
    },
    name: {
      type: "string",
      description: "name of the stoage (ex: CollatorState2)",
    },
  }).argv;
const main = async () => {
  console.log(
    `${argv.module}::${argv.name}: ${u8aToHex(
      u8aConcat(xxhashAsU8a(argv.module, 128), xxhashAsU8a(argv.name, 128))
    )}`
  );
};

main();
