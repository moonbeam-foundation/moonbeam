import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith } from "@moonwall/util";
import { countExtrinsics, createAccounts } from "../../../helpers/weights.js";

describeSuite({
  id: "D2988",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "delegateWithAutoCompound",
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions);

        await context.createBlock(
          randomAccounts.map((account, index) =>
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
        ).to.be.greaterThanOrEqual(17);
      },
    });
  },
});