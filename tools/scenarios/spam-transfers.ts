// This script is expected to run against a parachain network (using launch.ts script)

import { ALITH_PRIVATE_KEY } from "../utils/constants";
import { Keyring } from "@polkadot/api";

import yargs from "yargs";
import { getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { sendAllAndWaitLast } from "../utils/transactions";

type Account = ReturnType<Keyring["addFromUri"]>;

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    from: {
      type: "string",
      description: "Private key to transfer from",
      conflicts: ["to"],
    },
    to: {
      type: "string",
      description: "Private key to send to",
    },
    random: {
      type: "boolean",
      default: false,
      description: "Create random accounts",
      demandOption: true,
    },
    count: {
      type: "number",
      default: 1000,
      description: "Number of accounts",
      demandOption: true,
    },
    amount: {
      type: "number",
      default: 0.01,
      description: "Amount to transfer",
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
  const targetAccount = await keyring.addFromUri(argv.from || argv.to);
  console.log(`Using funds ${argv.from ? `from` : `to`} ${targetAccount.address}`);

  // Create a bunch of account using deterministic private key or random
  console.log(`${argv.random ? `Creating` : `Populating`} ${argv.count} accounts...`);
  const accounts = await Promise.all(
    new Array(argv.count).fill(0).map((_, i) => {
      if (argv.random) {
        return keyring.addFromUri(
          `0x${Math.floor(Math.random() * Number.MAX_SAFE_INTEGER)
            .toString()
            .padStart(64, "0")}`
        );
      }
      return keyring.addFromUri(`0x${(i + 100000).toString().padStart(64, "0")}`);
    })
  );

  let fromNonce = (await polkadotApi.rpc.system.accountNextIndex(targetAccount.address)).toNumber();

  console.log(`Creating ${argv.count} balance tranfers...`);
  const transferTxs = await Promise.all(
    accounts.map((account, index) => {
      if (argv.from) {
        return (
          polkadotApi.tx.balances
            // We need to multiple the float first to then convert to BigInt, 1000000 should be enough
            .transfer(account.address, BigInt(argv.amount * 1000000) * 10n ** 12n)
            .signAsync(targetAccount, { nonce: fromNonce + index })
        );
      } else {
        return (
          polkadotApi.tx.balances
            // We need to multiple the float first to then convert to BigInt, 1000000 should be enough
            .transfer(targetAccount.address, BigInt(argv.amount * 1000000) * 10n ** 12n)
            .signAsync(account, { nonce: -1 })
        );
      }
    })
  );

  // Send the transfer transactions and wait for the last one to finish
  await sendAllAndWaitLast(transferTxs);

  await polkadotApi.disconnect();
  console.log(`Finished`);
};

main();
