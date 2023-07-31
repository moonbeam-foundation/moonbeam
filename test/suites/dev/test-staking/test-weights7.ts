import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith } from "@moonwall/util";
import { chunk } from "../../../../tests/util/common.js";
import { countExtrinsics, createAccounts } from "../../../helpers/weights.js";
import { jumpRounds } from "../../../helpers/block.js";

describeSuite({
  id: "D2993",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "executeLeaveDelegators",
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith)
        );
        for (const randomAccountsChunk of chunk(randomAccounts, 17)) {
          await context.createBlock(
            randomAccountsChunk.map((account) =>
              context
                .polkadotJs()
                .tx.parachainStaking.delegateWithAutoCompound(
                  alith.address,
                  MIN_GLMR_DELEGATOR,
                  100,
                  maxTransactions,
                  maxTransactions,
                  0
                )
                .signAsync(account)
            )
          );
        }

        expect(
          (await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length,
          "Not all delegations were made, check batch size matches" +
            " delegateWithAutoCompound max qty per block"
        ).to.equal(maxTransactions);

        for (const randomAccountsChunk of chunk(randomAccounts, 21)) {
          await context.createBlock(
            randomAccountsChunk.map((account) =>
              context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(account)
            )
          );
        }

        await jumpRounds(context, 3);

        await context.createBlock(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.parachainStaking.executeLeaveDelegators(account.address, 1)
              .signAsync(account)
          )
        );

        /// Boilerplate to get the number of transactions

        const nameParts = expect.getState().currentTestName!.split(" ");
        const methodName = nameParts[nameParts.length - 1];
        const [numTransactions, weightUtil, proofUtil] = await countExtrinsics(
          context,
          methodName,
          log
        );

        expect(
          numTransactions,
          "Quantity of txns that fit inside block below previous max"
        ).to.be.greaterThanOrEqual(11);
      },
    });
  },
});
