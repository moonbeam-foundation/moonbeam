import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { baltathar, charleth, ethan } from "@moonwall/util";

describeSuite({
  id: "D013709",
  title: "Treasury proposal #8",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be approved by a non treasury collective vote",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

        const { result: rejectResult } = await context.createBlock(
          context
            .polkadotJs()
            .tx.openTechCommitteeCollective.propose(
              2,
              context.polkadotJs().tx.treasury.rejectProposal(0),
              1_000
            )
            .signAsync(charleth)
        );

        const councilProposalHash = rejectResult!.events
          .find(({ event: { method } }) => method.toString() == "Proposed")
          .event.data[2].toHex();

        // Charleth & Baltathar vote for against proposal and close it
        await context.createBlock([
          context
            .polkadotJs()
            .tx.openTechCommitteeCollective.vote(councilProposalHash, 0, true)
            .signAsync(charleth),
          context
            .polkadotJs()
            .tx.openTechCommitteeCollective.vote(councilProposalHash, 0, true)
            .signAsync(baltathar),
        ]);

        const { result: closeResult } = await context.createBlock(
          context
            .polkadotJs()
            .tx.openTechCommitteeCollective.close(
              councilProposalHash,
              0,
              {
                refTime: 800_000_000,
                proofSize: 64 * 1024,
              },
              1_000
            )
            .signAsync(baltathar),
          { expectEvents: [context.polkadotJs().events.openTechCommitteeCollective.Closed] }
        );

        expect(
          closeResult!.events.find((evt) =>
            context.polkadotJs().events.openTechCommitteeCollective.Executed.is(evt.event)
          ).event.data.result.asErr.isBadOrigin,
          "Proposal should be rejected due to wrong collective"
        ).toBe(true);

        expect(
          (await context.polkadotJs().query.treasury.proposals(0)).isSome,
          "The proposal must not have been deleted"
        ).toBe(true);
      },
    });
  },
});
