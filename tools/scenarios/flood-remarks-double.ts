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
      demandOption: true,
    },
    count: {
      type: "number",
      default: 1000,
      description: "Number of transactions",
      demandOption: true,
    },
    "transfer-initial-funds": {
      type: "boolean",
      default: true,
      description: "Should funds be transferered from Alice to those accounts",
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

  // Create a bunch of nominator using deterministic private key
  console.log(`Creating ${argv.count} nominators...`);
  const nominators = await Promise.all(
    new Array(argv.count).fill(0).map((_, i) => {
      return keyring.addFromUri(`0x${(i + 100000).toString().padStart(64, "0")}`);
    })
  );

  let fromNonce = (await polkadotApi.rpc.system.accountNextIndex(fromAccount.address)).toNumber();

  if (argv["transfer-initial-funds"]) {
    // Create transaction for 10 tokens tranfer to each nominator, from Alith
    console.log(`Creating ${argv.count} balance tranfers...`);
    const transferTxs = await Promise.all(
      nominators.map((nominator, index) =>
        polkadotApi.tx.balances
          .transfer(nominator.address, 10n ** 19n)
          .signAsync(fromAccount, { nonce: fromNonce + index })
      )
    );

    // Send the transfer transactions and wait for the last one to finish
    await sendAllAndWaitLast(transferTxs);
  }

  console.log(`Creating ${argv.count} remarks...`);
  const nominationTxs = [];
  // for each node
  // for each nominator (sequentially)

  for (let nominatorIndex = 0; nominatorIndex < nominators.length; nominatorIndex++) {
    const nominator = nominators[nominatorIndex];

    // Retrieve the nonce
    const nonce = (await polkadotApi.rpc.system.accountNextIndex(nominator.address)).toNumber();

    nominationTxs.push(await polkadotApi.tx.system.remark("ok").signAsync(nominator, { nonce }));
  }

  // Send the nomination transactions and wait for the last one to finish
  await sendAllAndWaitLast(nominationTxs);

  await polkadotApi.disconnect();
  console.log(`Finished`);
};

main();
