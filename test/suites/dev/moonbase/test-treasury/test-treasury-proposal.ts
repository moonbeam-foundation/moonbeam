import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D013801",
  title: "Treasury proposal #1",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be able to be approved by a non-council member",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt(), "new proposal should have been added").toBe(1n);

        await context.createBlock(
          context.polkadotJs().tx.treasury.approveProposal(0).signAsync(ethan),
          { expectEvents: [context.polkadotJs().events.system.ExtrinsicFailed] }
        );
        const approvals = await context.polkadotJs().query.treasury.approvals();

        expect(approvals.length, "No proposal must have been approved").to.equal(0);
      },
    });
  },
});
