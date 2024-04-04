import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import {
  jumpBlocks,
  expectEVMResult,
  extractRevertReason,
  createProposal,
  ConvictionVoting,
} from "../../../../helpers";

describeSuite({
  id: "D012835",
  title: "Precompiles - Ended proposal",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVoting: ConvictionVoting;

    beforeAll(async function () {
      // Whitelist caller is track 3
      proposalIndex = await createProposal({ context, track: "whitelistedcaller" });
      await context.createBlock(
        context.polkadotJs().tx.referenda.placeDecisionDeposit(proposalIndex),
        { allowFailures: false }
      );
      const alithAccount = await context.polkadotJs().query.system.account(ALITH_ADDRESS);

      convictionVoting = new ConvictionVoting(context);
      await convictionVoting
        .voteYes(proposalIndex, alithAccount.data.free.toBigInt() - 20n * 10n ** 18n, 1n)
        .block();
      // 20 minutes jump
      await jumpBlocks(context, (20 * 60) / 6);

      // Verifies the setup is correct
      const referendum = await context
        .polkadotJs()
        .query.referenda.referendumInfoFor(proposalIndex);
      expect(referendum.unwrap().isApproved).to.be.true;
    });

    beforeEach(async function () {
      convictionVoting = new ConvictionVoting(context);
    });

    // This and the next "it" and dependant on same state but this one is supposed to
    // revert and so not impact the proposal state
    it({
      id: "T01",
      title: `should failed to be removed without track info`,
      test: async function () {
        const block = await convictionVoting.withGas(2_000_000n).removeVote(proposalIndex).block();
        expectEVMResult(block.result!.events, "Revert", "Reverted");
        expect(await extractRevertReason(context, block.result!.hash)).to.contain("ClassNeeded");
      },
    });

    it({
      id: "T02",
      title: `should be removable by specifying the track`,
      test: async function () {
        const block = await convictionVoting.removeVoteForTrack(proposalIndex, 1).block();
        expectEVMResult(block.result!.events, "Succeed");
      },
    });
  },
});
