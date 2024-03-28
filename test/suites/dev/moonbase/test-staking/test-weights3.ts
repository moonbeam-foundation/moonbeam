import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith } from "@moonwall/util";
import { chunk, createAccounts, countExtrinsics } from "../../../../helpers";

describeSuite({
  id: "D013485",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "delegatorBondMore",
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions);

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
                .signAsync(account, { nonce: 0 })
            ),
            { allowFailures: false }
          );
        }

        expect(
          (await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length,
          "Not all delegations were made, check batch size matches" +
            " delegateWithAutoCompound max qty per block"
        ).to.equal(maxTransactions);

        await context.createBlock(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegatorBondMore(alith.address, 1000)
              .signAsync(account, { nonce: -1 })
          ),
          { allowFailures: false }
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
        ).to.be.greaterThanOrEqual(101);
      },
    });
  },
});
