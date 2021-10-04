// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import { exploreBlockRange, printBlockDetails } from "./utils/monitoring";

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
  const api = await getApiFor(argv);

  const toBlockNumber = argv.to || (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  const fromBlockNumber = argv.from;

  let totalExtrinsics = 0;
  let totalPercentages = 0;
  let blockCount = 0;
  let initialTimestamp = 0;
  let lastTimestamp = 0;
  let totalFees = 0n;
  await exploreBlockRange(
    api,
    { from: fromBlockNumber, to: toBlockNumber, concurrency: 5 },
    async (blockDetails) => {
      if (blockDetails.block.header.number.toNumber() % 100 == 0) {
        console.log(`${blockDetails.block.header.number.toNumber()}...`);
      }
      if (!initialTimestamp || blockDetails.blockTime < initialTimestamp) {
        initialTimestamp = blockDetails.blockTime;
      }
      if (!lastTimestamp || blockDetails.blockTime > lastTimestamp) {
        lastTimestamp = blockDetails.blockTime;
      }

      const fees = blockDetails.txWithEvents
        .filter(({ dispatchInfo }) => dispatchInfo.paysFee.isYes && !dispatchInfo.class.isMandatory)
        .reduce((p, { dispatchInfo, extrinsic, events, fee }) => {
          if (extrinsic.method.section == "ethereum") {
            return (
              p +
              (BigInt((extrinsic.method.args[0] as any).gasPrice) *
                dispatchInfo.weight.toBigInt()) /
                25000n
            );
          }
          return p + fee.partialFee.toBigInt();
        }, 0n);

      totalFees += fees;
      totalExtrinsics += blockDetails.txWithEvents.length;
      totalPercentages += blockDetails.weightPercentage;
      blockCount++;
      if (blockDetails.weightPercentage > 15) {
        printBlockDetails(blockDetails, {
          prefix: isKnownNetwork(argv.network)
            ? NETWORK_COLORS[argv.network](argv.network.padStart(10, " "))
            : undefined,
        });
      }
    }
  );
  console.log(
    `Total blocks: ${blockCount} (${Math.floor((lastTimestamp - initialTimestamp) / 1000)} secs), ${
      Math.floor((totalPercentages / blockCount) * 1000) / 1000
    }% fullness, ${Math.floor((totalExtrinsics / blockCount) * 1000) / 1000} tx/block (${
      Math.floor((totalExtrinsics / ((lastTimestamp - initialTimestamp) / 1000)) * 100) / 100
    } tx/s), ${Math.floor(Number(totalFees / 10n ** 15n)) / 1000} fees`
  );
};

main();
