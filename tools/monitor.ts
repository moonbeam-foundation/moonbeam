// This script is expected to run against a parachain network (using launch.ts script)

import { typesBundle } from "../moonbeam-types-bundle/dist";
import { ALITH_PRIVATE_KEY, BALTATHAR_PRIVATE_KEY } from "./utils/constants";
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";

import yargs from "yargs";
import { monitorBlocks, sendAllAndWaitLast } from "./utils/monitoring";
import { Extrinsic } from "./utils/types";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    url: {
      type: "string",
      default: "http://localhost:9944",
      description: "Websocket url",
    },
  })
  .demandOption(["url"]).argv;

const main = async () => {
  const wsProvider = new WsProvider(argv.url);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });

  await monitorBlocks(polkadotApi);
};

main();
