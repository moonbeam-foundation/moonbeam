import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith, baltathar, charleth, dorothy, ethan } from "@moonwall/util";

describeSuite({
  id: "D3208",
  title: "Treasury proposal #8",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be approved by COUNCIL collective vote",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.treasury.proposeSpend(10, baltathar.address).signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

        const { result: rejectResult } = await context.createBlock(
          context
            .polkadotJs()
            .tx.councilCollective.propose(
              2,
              context.polkadotJs().tx.treasury.rejectProposal(0),
              1_000
            )
            .signAsync(charleth)
        );
        const councilProposalHash = rejectResult!.events
          .find(({ event: { method } }) => method.toString() == "Proposed")
          .event.data[2].toHex();

        // Charleth & Dorothy vote for against proposal and close it
        await context.createBlock([
          context
            .polkadotJs()
            .tx.councilCollective.vote(councilProposalHash, 0, true)
            .signAsync(charleth),
          context
            .polkadotJs()
            .tx.councilCollective.vote(councilProposalHash, 0, true)
            .signAsync(dorothy),
        ]);

        const { result: closeResult } = await context.createBlock(
          context
            .polkadotJs()
            .tx.councilCollective.close(
              councilProposalHash,
              0,
              {
                refTime: 800_000_000,
                proofSize: 64 * 1024,
              },
              1_000
            )
            .signAsync(dorothy),
          { expectEvents: [context.polkadotJs().events.councilCollective.Closed] }
        );

        expect(
          closeResult!.events.find((evt) =>
            context.polkadotJs().events.councilCollective.Executed.is(evt.event)
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
