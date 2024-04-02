import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { baltathar, charleth, ethan } from "@moonwall/util";

describeSuite({
  id: "D013805",
  title: "Treasury proposal #4",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be rejected by less than half of the members of the treasury council",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt(), "new proposal should have been added").toBe(1n);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.propose(
              1, // Threshold of 1 is not 3/5 of collective
              context.polkadotJs().tx.treasury.rejectProposal(0),
              1_000
            )
            .signAsync(charleth),
          { expectEvents: [context.polkadotJs().events.treasuryCouncilCollective.Executed] }
        );

        const approvals = await context.polkadotJs().query.treasury.approvals();
        expect(approvals.length).to.equal(0, "No proposal should have been approved");
      },
    });
  },
});
