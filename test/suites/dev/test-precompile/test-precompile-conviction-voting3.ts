import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { createProposal } from "../../../helpers/voting.js";

describeSuite({
  id: "D2529-2",
  title: "Precompiles - Conviction on Root Track",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    beforeEach(async function () {
      proposalIndex = await createProposal(context);

      const rawTxn = await context.writePrecompile!({
        precompileName: "ConvictionVoting",
        functionName: "voteYes",
        args: [proposalIndex, 1n * 10n ** 18n, 1],
        rawTxOnly: true,
      });
      await context.createBlock(rawTxn);
      // Verifies the setup is correct
      const referendum = await context
        .polkadotJs()
        .query.referenda.referendumInfoFor(proposalIndex);
      expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
    });

    it({
      id: "T01",
      title: `should be removable`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeVote",
          args: [proposalIndex],
          rawTxOnly: true,
        });

        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Succeed");

        // Verifies the Subsrtate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T02",
      title: `should be removable by specifying the track`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeVoteForTrack",
          args: [proposalIndex, 0],
          rawTxOnly: true,
        });

        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Succeed");

        // Verifies the Subsrtate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });
  },
});
