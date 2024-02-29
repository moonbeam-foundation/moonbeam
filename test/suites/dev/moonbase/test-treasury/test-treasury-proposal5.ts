import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith, baltathar, ethan } from "@moonwall/util";

describeSuite({
  id: "D013706",
  title: "Treasury proposal #5",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be approvable by root",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt(), "new proposal should have been added").toBe(1n);

        // Root approve the proposal directly
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.treasury.approveProposal(0))
            .signAsync(alith),
          { expectEvents: [context.polkadotJs().events.sudo.Sudid] }
        );

        context.polkadotJs().query.system.events();

        const approvals = await context.polkadotJs().query.treasury.approvals();
        expect(approvals.length).to.equal(1, "One proposal should have been approved");
      },
    });
  },
});
