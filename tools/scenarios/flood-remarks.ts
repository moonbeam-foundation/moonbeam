// This script is expected to run against a parachain network (using launch.ts script)

import { ALITH_PRIVATE_KEY } from "../utils/constants";
import { Keyring } from "@polkadot/api";

import yargs from "yargs";
import { getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { sendAllAndWaitLast } from "../utils/transactions";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    from: {
      type: "string",
      description: "Private key to transfer from",
    },
    count: {
      type: "number",
      default: 1000,
      description: "Number of transactions",
      demandOption: true,
    },
  })
  .check(function (argv) {
    if (!argv.from && !argv.to) {
      argv.from = ALITH_PRIVATE_KEY;
    }
    return true;
  }).argv;

const main = async () => {
  const polkadotApi = await getMonitoredApiFor(argv.url || argv.network);

  const keyring = new Keyring({ type: "ethereum" });
  const fromAccount = await keyring.addFromUri(argv.from);
  console.log(`Using funds from ${fromAccount.address}`);

  let fromNonce = (await polkadotApi.rpc.system.accountNextIndex(fromAccount.address)).toNumber();

  console.log(`Creating ${argv.count} balance tranfers of ${argv.amount} Tokens...`);
  const transferTxs = await Promise.all(
    new Array(argv.count).fill(0).map((account, index) => {
      return polkadotApi.tx.system.remark("ok").signAsync(account, { nonce: fromNonce + index });
    })
  );

  // Send the transfer transactions and wait for the last one to finish
  await sendAllAndWaitLast(transferTxs);

  await polkadotApi.disconnect();
  console.log(`Finished`);
};

main();
