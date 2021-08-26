// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import moment from "moment-timezone";
import { listenBlocks } from "./utils/monitoring";

import { getApiFor, getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    "block-number": {
      type: "number",
      description: "Targeted block number",
      alias: "b",
    },
    timezone: {
      type: "string",
      default: "America/New_York",
      description: "MomentJs timezone format",
      alias: "t",
    },
  }).argv;

const main = async () => {
  const polkadotApi = await getApiFor(argv.url || argv.network);
  const targetBlockNumber = argv["block-number"];

  console.log(`Using timezone: ${argv.timezone}`);

  let lastBlockTime = "";
  await listenBlocks(polkadotApi, (blockDetails) => {
    const blockDiff = targetBlockNumber - blockDetails.block.header.number.toNumber();
    const targetMoment = moment.tz(argv.timezone).add(blockDiff * 12, "seconds");
    const targetTime = targetMoment.toString();
    if (lastBlockTime != targetTime) {
      console.log(`${targetBlockNumber}: ${targetMoment.toString()}`);
      lastBlockTime = targetTime;
    }
  });
};

main();
