import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D013703",
  title: "Treasury proposal #2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be able to be rejected by a non-council member",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt(), "new proposal should have been added").toBe(1n);

        await context.createBlock(
          context.polkadotJs().tx.treasury.rejectProposal(0).signAsync(ethan),
          { expectEvents: [context.polkadotJs().events.system.ExtrinsicFailed] }
        );
        expect(
          await context.polkadotJs().query.treasury.proposals(0),
          "The proposal should not have been deleted"
        ).not.equal(null);
      },
    });
  },
});
