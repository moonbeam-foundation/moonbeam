import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, baltathar, charleth, ethan } from "@moonwall/util";

describeSuite({
  id: "D013804",
  title: "Treasury proposal #3",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be rejected if three-fifths of the treasury council did not vote in favor",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.treasury.proposeSpend(17n * GLMR, baltathar.address)
            .signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt(), "new proposal should have been added").toBe(1n);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.propose(
              1, // Threshold of 1 is not 3/5 of collective
              context.polkadotJs().tx.treasury.approveProposal(0),
              1_000
            )
            .signAsync(charleth),
          { expectEvents: [context.polkadotJs().events.treasuryCouncilCollective.Executed] }
        );

        expect(await context.polkadotJs().query.treasury.proposals(0)).not.equal(
          null,
          "The proposal must not have been deleted"
        );
      },
    });
  },
});
