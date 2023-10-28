import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith } from "@moonwall/util";
import { chunk, createAccounts, countExtrinsics } from "../../../helpers";

const INITIAL_AMOUNT = 12n * MIN_GLMR_STAKING + 50n * GLMR;

describeSuite({
  id: "D2991",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "scheduleRevokeDelegation",
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions, INITIAL_AMOUNT);
        for (const randomAccountsChunk of chunk(randomAccounts, 17)) {
          await context.createBlock(
            randomAccountsChunk.map(
              (account) =>
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
                  .signAsync(account),
              { allowFailures: false }
            )
          );
        }

        expect(
          (await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length,
          "Not all delegations were made, check batch size matches" +
            " delegateWithAutoCompound max qty per block"
        ).to.equal(maxTransactions);

        await context.createBlock(
          randomAccounts.map(
            (account) =>
              context
                .polkadotJs()
                .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
                .signAsync(account, { nonce: -1 }),
            { allowFailures: false }
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
        ).to.be.greaterThanOrEqual(103);
      },
    });
  },
});
