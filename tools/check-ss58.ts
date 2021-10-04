// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import { exploreBlockRange } from "./utils/monitoring";

import { DispatchInfo } from "@polkadot/types/interfaces";
import Keyring from "@polkadot/keyring";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ss58: {
      type: "string",
      description: "Address to decode",
      demandOption: true,
    },
  }).argv;

const main = async () => {
  const keyring = new Keyring();

  console.log(Buffer.from(keyring.decodeAddress(argv.ss58)).toString("hex"));
};

main();
