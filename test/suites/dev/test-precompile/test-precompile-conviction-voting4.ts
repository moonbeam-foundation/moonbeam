import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_PRIVATE_KEY, GLMR } from "@moonwall/util";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { createProposal } from "../../../helpers/voting.js";

describeSuite({
  id: "D2529-3",
  title: "Precompiles - Conviction on General Admin Track",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    beforeEach(async function () {
      proposalIndex = await createProposal(context, "generaladmin");

      const rawTxn = await context.writePrecompile!({
        precompileName: "ConvictionVoting",
        functionName: "voteYes",
        args: [proposalIndex, GLMR, 1],
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
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T02",
      title: `should be removable using self removeOtherVote`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeOtherVote",
          args: [ALITH_ADDRESS, 2, proposalIndex],
          rawTxOnly: true,
        });

        // general_admin is track 2
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Succeed");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T03",
      title: `should be removable by specifying the track general_admin`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeVoteForTrack",
          args: [proposalIndex, 2],
          rawTxOnly: true,
        });

        // general_admin is track 2
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Succeed");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T04",
      title: `should not be removable by specifying the wrong track`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeVoteForTrack",
          args: [proposalIndex, 0],
          gas: 2_000_000n,
          rawTxOnly: true,
        });

        // general_admin is track 2
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Revert");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T05",
      title: `should not be removable by someone else during voting time`,
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "ConvictionVoting",
          functionName: "removeOtherVote",
          args: [ALITH_ADDRESS, 2, proposalIndex],
          rawTxOnly: true,
          gas: 2_000_000n,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });

        // general_admin is track 2
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Revert");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });
  },
});
