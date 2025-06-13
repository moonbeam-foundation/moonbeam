import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { type KeyringPair, alith } from "@moonwall/util";
import { createAccounts, chunk, expectSubstrateEvents } from "../../../../helpers";

describeSuite({
  id: "D020605",
  title: "Conviction Voting - Undelegate Weight Fit",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let randomAccounts: KeyringPair[] = [];
    beforeAll(async () => {
      randomAccounts = await createAccounts(context, 100);

      // delegate 100 accounts
      for (const randomChunk of chunk(randomAccounts, 10)) {
        await context.createBlock(
          randomChunk.map((account) =>
            context
              .polkadotJs()
              .tx.convictionVoting.delegate(1, alith.address, 1, 1000000000000000000n)
              .signAsync(account)
          )
        );
      }
    });

    it({
      id: "T01",
      title: "should undelegate at least 25 txs in a block",
      test: async function () {
        const blockResult = await context.createBlock(
          randomAccounts.map((account) =>
            context.polkadotJs().tx.convictionVoting.undelegate(1).signAsync(account)
          )
        );

        const undelegatedEvents = expectSubstrateEvents(
          blockResult,
          "convictionVoting",
          "Undelegated"
        );
        expect(undelegatedEvents.length).to.be.greaterThanOrEqual(25);
      },
    });
  },
});
