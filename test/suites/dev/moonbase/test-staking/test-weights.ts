import "@moonbeam-network/api-augment";
import { GLMR, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, describeSuite, expect } from "moonwall";
import { createAccounts, countExtrinsics } from "../../../../helpers";

const INITIAL_AMOUNT = 12n * MIN_GLMR_STAKING + 50n * GLMR;

describeSuite({
  id: "D023385",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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
        const [numTransactions] = await countExtrinsics(context, methodName);

        expect(
          numTransactions,
          "Quantity of txns that fit inside block below previous max"
        ).to.be.greaterThanOrEqual(79);
      },
    });

    it({
      id: "T02",
      title: "delegateWithAutoCompound",
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions, INITIAL_AMOUNT);
        const txns = await Promise.all(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegateWithAutoCompound(
                alith.address,
                MIN_GLMR_DELEGATOR,
                0,
                maxTransactions,
                0,
                0
              )
              .signAsync(account)
          )
        );
        await context.createBlock(txns, { allowFailures: false });

        /// Boilerplate to get the number of transactions
        const nameParts = expect.getState().currentTestName!.split(" ");
        const methodName = nameParts[nameParts.length - 1];
        const [numTransactions] = await countExtrinsics(context, methodName);

        expect(
          numTransactions,
          "Quantity of txns that fit inside block below previous max"
        ).to.be.greaterThanOrEqual(9);
      },
    });
  },
});
