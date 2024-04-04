import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { baltathar, charleth, dorothy, ethan } from "@moonwall/util";

describeSuite({
  id: "D013802",
  title: "Treasury proposal #10",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be rejected if the half of the treasury council voted against it",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");
        console.log("proposalCount.toBigInt()", proposalCount.toBigInt());

        // Charleth proposed that the council reject the treasury proposal
        // (and therefore implicitly votes for)
        const { result: proposalResult } = await context.createBlock(
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.propose(
              2,
              context.polkadotJs().tx.treasury.rejectProposal(0),
              1_000
            )
            .signAsync(charleth)
        );

        const councilProposalHash = proposalResult!.events
          .find(({ event: { method } }) => method.toString() == "Proposed")!
          .event.data[2].toHex();
        console.log("councilProposalHash", councilProposalHash);

        // Charleth & Dorothy vote for against proposal and close it
        await context.createBlock([
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.vote(councilProposalHash, 0, true)
            .signAsync(charleth),
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.vote(councilProposalHash, 0, true)
            .signAsync(dorothy),
        ]);
        console.log("Create block");

        const { result: closeResult } = await context.createBlock(
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.close(
              councilProposalHash,
              0,
              {
                refTime: 800_000_000,
                proofSize: 64 * 1024,
              },
              1_000
            )
            .signAsync(dorothy),
          {
            expectEvents: [
              context.polkadotJs().events.treasuryCouncilCollective.Closed,
              context.polkadotJs().events.treasuryCouncilCollective.Approved,
              context.polkadotJs().events.treasury.Rejected,
              context.polkadotJs().events.balances.Slashed,
            ],
          }
        );
        console.log("Create block 2");

        expect(
          closeResult!.events.find((evt) =>
            context.polkadotJs().events.treasuryCouncilCollective.Executed.is(evt.event)
          ).event.data.result.isOk
        ).toBe(true);
        console.log("closeResult");

        expect((await context.polkadotJs().query.treasury.proposals(0)).toHuman()).to.equal(
          null,
          "The proposal must have been deleted"
        );
        console.log("deleted done");
      },
    });
  },
});
