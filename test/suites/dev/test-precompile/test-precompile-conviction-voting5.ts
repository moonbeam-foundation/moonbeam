import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { jumpBlocks } from "../../../helpers/block.js";
import { expectEVMResult, extractRevertReason } from "../../../helpers/eth-transactions.js";
import { createProposal } from "../../../helpers/voting.js";

describeSuite({
  id: "D2529-4",
  title: "Precompiles - Ended proposal",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;

    beforeAll(async function () {
      // Whitelist caller is track 3
      proposalIndex = await createProposal(context, "whitelistedcaller");
      await context.createBlock(
        context.polkadotJs().tx.referenda.placeDecisionDeposit(proposalIndex),
        { allowFailures: false }
      );
      const alithAccount = await context.polkadotJs().query.system.account(ALITH_ADDRESS);

      const rawTxn = await context.writePrecompile!({
        precompileName: "ConvictionVoting",
        functionName: "voteYes",
        args: [proposalIndex, alithAccount.data.free.toBigInt() - 20n * 10n ** 18n, 1],
        rawTxOnly: true,
      });

      await context.createBlock(
        rawTxn,

        { allowFailures: false }
      );
      // 20 minutes jump
      await jumpBlocks(context, (20 * 60) / 12);

      // Verifies the setup is correct
      const referendum = await context
        .polkadotJs()
        .query.referenda.referendumInfoFor(proposalIndex);
      expect(referendum.unwrap().isApproved).to.be.true;
    });

    // This and the next "it" and dependant on same state but this one is supposed to
    // revert and so not impact the proposal state
    it({
      id: "T01",
      title: `should failed to be removed without track info`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeVote",
          args: [proposalIndex],
          rawTxOnly: true,
          gas: 2_000_000n,
        });
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Revert", "Reverted");
        expect(await extractRevertReason(context, block.result!.hash)).to.contain("ClassNeeded");
      },
    });

    it({
      id: "T02",
      title: `should be removable by specifying the track`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeVoteForTrack",
          args: [proposalIndex, 1],
          rawTxOnly: true,
        });
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Succeed");
      },
    });
  },
});
