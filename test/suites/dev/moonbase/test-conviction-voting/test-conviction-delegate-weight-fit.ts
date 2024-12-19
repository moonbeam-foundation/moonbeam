import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { type KeyringPair, alith } from "@moonwall/util";
import { createAccounts, expectSubstrateEvents } from "../../../../helpers";

describeSuite({
  id: "D010702",
  title: "Conviction Voting - Delegate Weight Fit",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let randomAccounts: KeyringPair[] = [];
    beforeAll(async () => {
      randomAccounts = await createAccounts(context, 100);
    });

    it({
      id: "T01",
      title: "should delegate at least 25 txs in a block",
      test: async function () {
        const blockResult = await context.createBlock(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.convictionVoting.delegate(1, alith.address, 1, 1000000000000000000n)
              .signAsync(account)
          )
        );

        const delegatedEvents = expectSubstrateEvents(blockResult, "convictionVoting", "Delegated");
        expect(delegatedEvents.length).to.be.greaterThanOrEqual(25);
      },
    });
  },
});
