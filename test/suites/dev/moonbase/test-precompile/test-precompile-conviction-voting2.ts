import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { expectEVMResult, createProposal, ConvictionVoting } from "../../../../helpers";

const CONVICTION_VALUES = [0n, 1n, 2n, 3n, 4n, 5n, 6n];

describeSuite({
  id: "D022818",
  title: "Precompiles - Conviction",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVoting: ConvictionVoting;

    beforeEach(async function () {
      convictionVoting = new ConvictionVoting(context);
      proposalIndex = await createProposal({ context });
    });

    for (const conviction of CONVICTION_VALUES) {
      it({
        id: "T01",
        title: `should allow to vote with conviction x${conviction}`,
        test: async function () {
          const block = await convictionVoting
            .voteYes(proposalIndex, 1n * 10n ** 18n, conviction)
            .block();

          expectEVMResult(block.result!.events, "Succeed");

          // Verifies the substrate side
          const referendum = await context
            .polkadotJs()
            .query.referenda.referendumInfoFor(proposalIndex);
          expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
            1n * 10n ** 17n * (conviction === 0n ? 1n : conviction * 10n)
          );
        },
      });
    }
  },
});
