// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import { listenBlocks } from "../utils/monitoring";

import { getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { TREASURY_ADDRESS } from "../utils/constants";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
  }).argv;

const main = async () => {
  const api = await getMonitoredApiFor(argv);

  let lastTreasuryBalance = 0n;
  await listenBlocks(api, false, async (blockDetails) => {
    const blockNumber = blockDetails.block.header.number.toNumber();
    const blockHash = blockDetails.block.hash.toHex();
    const treasuryBalance = await api.query.system.account.at(blockHash, TREASURY_ADDRESS);
    const fees = blockDetails.txWithEvents
      .filter(({ dispatchInfo }) => dispatchInfo.paysFee.isYes && !dispatchInfo.class.isMandatory)
      .reduce((p, { dispatchInfo, extrinsic, events, fee }) => {
        if (extrinsic.method.section == "ethereum") {
          return (
            p +
            (BigInt((extrinsic.method.args[0] as any).gasPrice) * dispatchInfo.weight.toBigInt()) /
              25000n
          );
        }
        return p + fee.partialFee.toBigInt();
      }, 0n);

    console.log(
      `                         [${fees
        .toString()
        .padStart(20)}💰][Tresaury: ${treasuryBalance.data.free
        .toBigInt()
        .toString()
        .padStart(20)} (+${treasuryBalance.data.free.toBigInt() - lastTreasuryBalance})]`
    );

    lastTreasuryBalance = treasuryBalance.data.free.toBigInt();
  });
};

main();
