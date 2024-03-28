import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, baltathar, charleth, dorothy, ethan } from "@moonwall/util";

describeSuite({
  id: "D013810",
  title: "Treasury proposal #9",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be approved if the three fifths of the treasury council voted for it",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.treasury.proposeSpend(17n * GLMR, baltathar.address)
            .signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.treasuryCouncilCollective.propose(
              2,
              context.polkadotJs().tx.treasury.approveProposal(0),
              1_000
            )
            .signAsync(charleth),
          { expectEvents: [context.polkadotJs().events.treasuryCouncilCollective.Proposed] }
        );
        const proposalHash = result!.events
          .find(({ event: { method } }) => method.toString() == "Proposed")
          .event.data[2].toHex();

        // Charleth & Dorothy vote for this proposal and close it
        const { result: closeResult } = await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.treasuryCouncilCollective.vote(proposalHash, 0, true)
              .signAsync(charleth),
            context
              .polkadotJs()
              .tx.treasuryCouncilCollective.vote(proposalHash, 0, true)
              .signAsync(dorothy, { nonce: 0 }),
            context
              .polkadotJs()
              .tx.treasuryCouncilCollective.close(
                proposalHash,
                0,
                {
                  refTime: 800_000_000,
                  proofSize: 64 * 1024,
                },
                1_000
              )
              .signAsync(dorothy, { nonce: 1 }),
          ],
          {
            expectEvents: [
              context.polkadotJs().events.treasuryCouncilCollective.Closed,
              context.polkadotJs().events.treasuryCouncilCollective.Approved,
              context.polkadotJs().events.treasuryCouncilCollective.Executed,
            ],
          }
        );

        expect(
          closeResult![closeResult!.length - 1].events.find((evt) =>
            context.polkadotJs().events.treasuryCouncilCollective.Executed.is(evt.event)
          ).event.data.result.isOk
        ).toBe(true);

        const approvals = await context.polkadotJs().query.treasury.approvals();
        expect(approvals.length).to.equal(1, "one proposal should have been approved");
      },
    });
  },
});
