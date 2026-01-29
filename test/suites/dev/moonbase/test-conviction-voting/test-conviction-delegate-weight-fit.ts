import "@moonbeam-network/api-augment";
import { alith, beforeAll, describeSuite, expect } from "moonwall";
import type { KeyringPair } from "@polkadot/keyring/types";
import { createAccounts, expectSubstrateEvents } from "../../../../helpers";

describeSuite({
  id: "D020602",
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

        const delegatedEvents = expectSubstrateEvents(blockResult as any, "convictionVoting", "Delegated");
        expect(delegatedEvents.length).to.be.greaterThanOrEqual(25);
      },
    });
  },
});
