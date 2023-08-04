import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  GLMR,
} from "@moonwall/util";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { expectSubstrateEvent } from "../../../helpers/expect.js";
import { cancelProposal, createProposal } from "../../../helpers/voting.js";
import { decodeEventLog } from "viem";

describeSuite({
  id: "D2529-3",
  title: "Precompiles - Conviction on General Admin Track",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVotingAbi: Abi;

    beforeAll(async function () {
      const { abi } = fetchCompiledContract("ConvictionVoting");
      convictionVotingAbi = abi;
    });

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
        const trackId = 2;
        // Cancel the proposal
        await cancelProposal(context, proposalIndex);
        const rawTxn = await context.writePrecompile!({
          privateKey: ETHAN_PRIVATE_KEY,
          precompileName: "ConvictionVoting",
          functionName: "removeOtherVote",
          args: [ALITH_ADDRESS, trackId, proposalIndex],
          rawTxOnly: true,
        });

        // general_admin is track 2
        const block = await context.createBlock(rawTxn);
        expectEVMResult(block.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: convictionVotingAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;

        expect(evmLog.eventName, "Wrong event").to.equal("VoteRemovedOther");
        expect(evmLog.args.caller).to.equal(ETHAN_ADDRESS);
        expect(evmLog.args.target).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.trackId).to.equal(trackId);

        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().isCancelled).to.equal(true);
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
