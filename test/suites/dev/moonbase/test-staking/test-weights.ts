import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith } from "@moonwall/util";
import { createAccounts, countExtrinsics } from "../../../../helpers";

const INITIAL_AMOUNT = 12n * MIN_GLMR_STAKING + 50n * GLMR;

describeSuite({
  id: "D013483",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "joinCandidates",
      test: async () => {
        const maxTransactions = 100;
        const randomAccounts = await createAccounts(context, maxTransactions, INITIAL_AMOUNT);

        await context.createBlock(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, maxTransactions)
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
        ).to.be.greaterThanOrEqual(79);
      },
    });

    it({
      id: "T02",
      title: "delegate",
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions, INITIAL_AMOUNT);
        const txns = await Promise.all(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, maxTransactions, 0)
              .signAsync(account)
          )
        );
        await context.createBlock(txns, { allowFailures: false });

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
        ).to.be.greaterThanOrEqual(9);
      },
    });
  },
});
