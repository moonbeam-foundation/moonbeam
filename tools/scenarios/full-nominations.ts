// This script is expected to run against a parachain network (using launch.ts script)

import { ALITH_PRIVATE_KEY, BALTATHAR_PRIVATE_KEY } from "../utils/constants";
import { Keyring } from "@polkadot/api";

import yargs from "yargs";
import { getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { sendAllAndWaitLast } from "../utils/transactions";

type Account = ReturnType<Keyring["addFromUri"]>;

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    nominators: {
      type: "number",
      default: 2000,
      description: "Number of nominators",
    },
    "transfer-initial-funds": {
      type: "boolean",
      default: true,
      description: "Should funds be transferered from Alice to those accounts",
    },
  }).argv;

const main = async () => {
  const polkadotApi = await getMonitoredApiFor(argv.url || argv.network);

  const keyring = new Keyring({ type: "ethereum" });
  const alith = await keyring.addFromUri(ALITH_PRIVATE_KEY);
  const baltathar = await keyring.addFromUri(BALTATHAR_PRIVATE_KEY);

  // Create a bunch of nominator using deterministic private key
  console.log(`Creating ${argv.nominators} nominators...`);
  const nominators = await Promise.all(
    new Array(argv.nominators).fill(0).map((_, i) => {
      return keyring.addFromUri(`0x${(i + 100000).toString().padStart(64, "0")}`);
    })
  );

  const node1 = alith;
  const node2 = baltathar;
  let aliceNonce = (await polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();

  if (argv["transfer-initial-funds"]) {
    // Create transaction for 10 tokens tranfer to each nominator, from Alith
    console.log(`Creating ${argv.nominators} balance tranfers...`);
    const transferTxs = await Promise.all(
      nominators.map((nominator, index) =>
        polkadotApi.tx.balances
          .transfer(nominator.address, 10n ** 19n)
          .signAsync(alith, { nonce: aliceNonce + index })
      )
    );

    // Send the transfer transactions and wait for the last one to finish
    await sendAllAndWaitLast(transferTxs);
  }

  const nodes = [node1, node2];
  console.log(`Creating ${nodes.length * argv.nominators} nominations...`);
  const nominationTxs = (
    await Promise.all(
      // for each node
      nodes.map(async (node, nodeIndex) => {
        const transactions: SubmittableExtrinsic[] = [];
        // for each nominator (sequentially)
        for (let nominatorIndex = 0; nominatorIndex < nominators.length; nominatorIndex++) {
          const nominator = nominators[nominatorIndex];

          // Retrieve the nonce
          const nonce = (
            await polkadotApi.rpc.system.accountNextIndex(nominator.address)
          ).toNumber();

          // Creates and Adds the nomination transaction (4 token)
          transactions.push(
            await polkadotApi.tx.parachainStaking
              .nominate(node.address, 4n * 10n ** 18n, nominatorIndex + nodeIndex, 2)
              .signAsync(nominator, { nonce: nonce + nodeIndex })
          );
        }
        return transactions;
      })
    )
  ).flat();

  // Send the nomination transactions and wait for the last one to finish
  await sendAllAndWaitLast(nominationTxs);

  await polkadotApi.disconnect();
  console.log(`Finished`);
};

main();
