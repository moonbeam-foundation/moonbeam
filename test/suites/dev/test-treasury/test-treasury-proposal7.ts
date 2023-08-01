import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith, baltathar, charleth, dorothy, ethan } from "@moonwall/util";
import { Result } from "@polkadot/types";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";

describeSuite({
  id: "D3207",
  title: "Treasury proposal #7",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not be approved by COUNCIL collective vote",
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
            .tx.councilCollective.propose(
              2,
              context.polkadotJs().tx.treasury.approveProposal(0),
              1_000
            )
            .signAsync(charleth)
        );
        const proposalHash = result!.events
          .find(({ event: { method } }) => method.toString() == "Proposed")
          .event.data[2].toHex();

        // Charleth and Dorothy vote for proposal to approve
        const { result: result2 } = await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.councilCollective.vote(proposalHash, 0, true)
              .signAsync(charleth),
            context
              .polkadotJs()
              .tx.councilCollective.vote(proposalHash, 0, true)
              .signAsync(dorothy, { nonce: 0 }),
            context
              .polkadotJs()
              .tx.councilCollective.close(
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
          { expectEvents: [context.polkadotJs().events.councilCollective.Closed] }
        );

        expect(
          result2![result2!.length - 1].events.find((evt) =>
            context.polkadotJs().events.councilCollective.Executed.is(evt.event)
          ).event.data.result.asErr.isBadOrigin,
          "Proposal should be rejected due to wrong collective"
        ).toBe(true);

        const approvals = await context.polkadotJs().query.treasury.approvals();
        expect(approvals.length).to.equal(0, "No proposal should have been approved");
      },
    });
  },
});
