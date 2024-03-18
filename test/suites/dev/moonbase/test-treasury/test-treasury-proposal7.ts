import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, baltathar, charleth, ethan } from "@moonwall/util";

describeSuite({
  id: "D013708",
  title: "Treasury proposal #7",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be approved by the wrong collective vote",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.treasury.proposeSpend(17n * GLMR, baltathar.address)
            .signAsync(ethan)
        );

        const proposalCount = await context.polkadotJs().query.treasury.proposalCount();
        expect(proposalCount.toBigInt()).to.equal(1n, "new proposal should have been added");

        // Charleth submits council proposal to approve
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.openTechCommitteeCollective.propose(
              2,
              context.polkadotJs().tx.treasury.approveProposal(0),
              1_000
            )
            .signAsync(charleth)
        );
        const proposalHash = result!.events
          .find(({ event: { method } }) => method.toString() == "Proposed")
          .event.data[2].toHex();

        // Charleth and Baltahar vote for proposal to approve
        const { result: result2 } = await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.openTechCommitteeCollective.vote(proposalHash, 0, true)
              .signAsync(charleth),
            context
              .polkadotJs()
              .tx.openTechCommitteeCollective.vote(proposalHash, 0, true)
              .signAsync(baltathar, { nonce: 0 }),
            context
              .polkadotJs()
              .tx.openTechCommitteeCollective.close(
                proposalHash,
                0,
                {
                  refTime: 800_000_000,
                  proofSize: 64 * 1024,
                },
                1_000
              )
              .signAsync(baltathar, { nonce: 1 }),
          ],
          { expectEvents: [context.polkadotJs().events.openTechCommitteeCollective.Closed] }
        );

        expect(
          result2![result2!.length - 1].events.find((evt) =>
            context.polkadotJs().events.openTechCommitteeCollective.Executed.is(evt.event)
          ).event.data.result.asErr.isBadOrigin,
          "Proposal should be rejected due to wrong collective"
        ).toBe(true);

        const approvals = await context.polkadotJs().query.treasury.approvals();
        expect(approvals.length).to.equal(0, "No proposal should have been approved");
      },
    });
  },
});
