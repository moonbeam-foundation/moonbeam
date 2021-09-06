// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";

import { getApiFor, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    at: {
      type: "number",
      description: "Block number to look into",
      demandOption: true,
    },
  }).argv;

const main = async () => {
  const api = await getApiFor(argv.url || argv.network);

  const blockNumber = argv.at;
  const blockHash = (await api.rpc.chain.getBlockHash(blockNumber)).toString();
  const records = await api.query.system.events.at(blockHash);

  console.log(`========= Checking block ${blockNumber - 1}...(${blockHash})`);

  let total = 0n;
  records.forEach(({ event }, index) => {
    if (event.section == "parachainStaking" && event.method == "Rewarded") {
      const [acc, amount] = event.data as any;
      console.log(`#${blockNumber} - ${acc.toString()}: ${amount.toHuman()}`);
      total += amount.toBigInt();
    }
  });
  console.log(`\n#${blockNumber} Total : ${Number(total / 10n ** 15n) / 1000} MOVRs`);
};

main();
