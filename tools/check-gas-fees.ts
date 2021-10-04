// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import { exploreBlockRange } from "./utils/monitoring";

import { DispatchInfo } from "@polkadot/types/interfaces";

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

const POWER = 10n ** (18n - BigInt(2));
const DECIMAL_POWER = 10 ** 2;
const printMOVRs = (value: bigint) => {
  if (2 > 0) {
    return (Number(value / POWER) / DECIMAL_POWER).toFixed(2).padStart(6 + 2, " ");
  }
  return (value / POWER).toString().padStart(6, " ");
};

const main = async () => {
  const api = await getApiFor(argv);

  const toBlockNumber = argv.to || (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  const fromBlockNumber = argv.from;

  console.log(`========= Checking block ${fromBlockNumber}...${toBlockNumber}`);
  let allBlocksTotal = 0n;
  let blockCount = 1;
  await exploreBlockRange(
    api,
    { from: fromBlockNumber, to: toBlockNumber, concurrency: 5 },
    async (blockDetails) => {
      blockCount++;
      let total = 0n;

      blockDetails.txWithEvents.forEach(({ events, extrinsic }) => {
        const weightPrice =
          extrinsic.method.section == "ethereum"
            ? (extrinsic.method.args[0] as any).gasPrice.toBigInt() / 25000n
            : 0n;

        events.forEach((event, index) => {
          if (event.section == "system" && event.method == "ExtrinsicSuccess") {
            const dispatchInfo = event.data[0] as DispatchInfo;
            if (dispatchInfo.paysFee.isYes) {
              total += dispatchInfo.weight.toBigInt() * weightPrice;
              console.log(dispatchInfo.weight.toBigInt() * weightPrice);
            }
          }
        });
      });
      allBlocksTotal += total;
      console.log(
        `\n#${blockDetails.block.header.number} Fees : ${
          Number(total / 10n ** 11n) / 10000000
        } MOVRs`
      );
    }
  );
  console.log(
    `Total blocks: ${blockCount}, ${printMOVRs(
      allBlocksTotal / BigInt(blockCount)
    )}/block, ${printMOVRs(allBlocksTotal)} Total`
  );
};

main();
