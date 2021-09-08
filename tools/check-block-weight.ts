// This script is expected to run against a parachain network (using launch.ts script)
import chalk from "chalk";
import yargs from "yargs";
import {
  exploreBlockRange,
  listenBestBlocks,
  listenFinalizedBlocks,
  printBlockDetails,
} from "./utils/monitoring";

import { getApiFor, isKnownNetwork, NETWORK_COLORS, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    from: {
      type: "number",
      description: "from block number (included)",
      demandOption: true,
    },
    to: {
      type: "number",
      description: "to block number (included)",
    },
  }).argv;

const main = async () => {
  const nameOrUrl = argv.url || argv.network;
  const api = await getApiFor(nameOrUrl);

  const toBlockNumber = argv.to || (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  const fromBlockNumber = argv.from;

  let totalExtrinsics = 0;
  let totalPercentages = 0;
  let blockCount = 0;
  await exploreBlockRange(
    api,
    { from: fromBlockNumber, to: toBlockNumber, concurrency: 5 },
    async (blockDetails) => {
      totalExtrinsics += blockDetails.txWithEvents.length;
      totalPercentages += blockDetails.weightPercentage;
      blockCount++;
      if (blockDetails.weightPercentage > 15) {
        printBlockDetails(blockDetails, {
          prefix: isKnownNetwork(nameOrUrl)
            ? NETWORK_COLORS[nameOrUrl](nameOrUrl.padStart(10, " "))
            : undefined,
        });
      }
    }
  );
  console.log(
    `Total blocks: ${blockCount}, ${
      Number((totalPercentages / blockCount) * 100) / 100
    }% fullness, ${Number((totalExtrinsics / blockCount) * 100) / 100} extrinsics`
  );
};

main();
