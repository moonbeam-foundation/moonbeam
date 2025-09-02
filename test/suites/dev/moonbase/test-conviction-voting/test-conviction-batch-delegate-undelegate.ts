import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { type KeyringPair, alith } from "@moonwall/util";
import { createAccounts, expectSubstrateEvents } from "../../../../helpers";

describeSuite({
  id: "D020601",
  title: "Conviction Voting - Batch Delegate and Undelegate",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let randomAccounts: KeyringPair[] = [];

    beforeAll(async () => {
      randomAccounts = await createAccounts(context, 10);
      let alithNonce = await context
        .viem("public")
        .getTransactionCount({ address: alith.address as `0x{string}` });

      // delegate to first 5 accounts from alice with different tracks
      const blockResult = await context.createBlock([
        ...randomAccounts
          .slice(0, 5)
          .map((account, index) =>
            context
              .polkadotJs()
              .tx.convictionVoting.delegate(index, account.address, 1, 1000000000000000000n)
              .signAsync(alith, { nonce: alithNonce++ })
          ),
      ]);
      const delegatedEvents = expectSubstrateEvents(blockResult, "convictionVoting", "Delegated");
      expect(delegatedEvents.length).to.equal(5);
    });

    it({
      id: "T01",
      title: "should batch delegate and undelegate 5 of each txs in a block",
      test: async function () {
        const blockResult = await context.createBlock(
          context
            .polkadotJs()
            .tx.utility.batchAll([
              ...randomAccounts
                .slice(0, 5)
                .map((_, index) => context.polkadotJs().tx.convictionVoting.undelegate(index)),
              ...randomAccounts
                .slice(5)
                .map((account, index) =>
                  context
                    .polkadotJs()
                    .tx.convictionVoting.delegate(index, account.address, 1, 1000000000000000000n)
                ),
            ])
            .signAsync(alith)
        );

        const undelegatedEvents = expectSubstrateEvents(
          blockResult,
          "convictionVoting",
          "Undelegated"
        );
        const delegatedEvents = expectSubstrateEvents(blockResult, "convictionVoting", "Delegated");

        expect(undelegatedEvents.length).to.equal(5);
        expect(delegatedEvents.length).to.equal(5);
      },
    });
  },
});
