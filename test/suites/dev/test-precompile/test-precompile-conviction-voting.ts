import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  PRECOMPILE_CONVICTION_VOTING_ADDRESS,
  createViemTransaction,
} from "@moonwall/util";
import { Abi, decodeEventLog, encodeFunctionData } from "viem";
import { expectEVMResult, extractRevertReason } from "../../../helpers/eth-transactions.js";
import { expectSubstrateEvent } from "../../../helpers/expect.js";
import { createProposal } from "../../../helpers/voting.js";

describeSuite({
  id: "D2529",
  title: "Precompiles - Conviction Voting precompile",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVotingAbi: Abi;

    beforeAll(async function () {
      const { abi } = fetchCompiledContract("ConvictionVoting");
      convictionVotingAbi = abi;
    });

    beforeEach(async function () {
      proposalIndex = await createProposal(context);
    });

    it({
      id: "T01",
      title: "should allow to vote yes for a proposal",
      test: async function () {
        const rawTx = await createViemTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "voteYes",
            args: [proposalIndex, 1n * 10n ** 18n, 1],
          }),
        });
        const block = await context.createBlock(rawTx);

        // Verifies the EVM Side
        expectEVMResult(block.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: convictionVotingAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;

        expect(evmLog.eventName, "Wrong event").to.equal("Voted");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.aye).to.equal(true);
        expect(BigInt(evmLog.args.voteAmount)).to.equal(1n * 10n ** 18n);
        expect(evmLog.args.conviction).to.equal(1);

        // Verifies the Substrate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T02",
      title: "should allow to vote no for a proposal",
      test: async function () {
        const block = await context.createBlock(
          await createViemTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteNo",
              args: [proposalIndex, 1n * 10n ** 18n, 1],
            }),
          })
        );

        expectEVMResult(block.result!.events, "Succeed");
        const { data } = await expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: convictionVotingAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;

        expect(evmLog.eventName, "Wrong event").to.equal("Voted");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.aye).to.equal(false);
        expect(BigInt(evmLog.args.voteAmount)).to.equal(1n * 10n ** 18n);
        expect(evmLog.args.conviction).to.equal(1);

        // Verifies the Subsrtate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T03",
      title: "should allow to replace yes by a no",
      test: async function () {
        const block1 = await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteYes",
              args: [proposalIndex, 1n * 10n ** 18n, 1],
            }),
          })
        );
        expectEVMResult(block1.result!.events, "Succeed");

        const block2 = await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteNo",
              args: [proposalIndex, 1n * 10n ** 18n, 1],
            }),
          })
        );
        expectEVMResult(block2.result!.events, "Succeed");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
        expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T04",
      title: "should fail to vote for the wrong proposal",
      test: async function () {
        const block = await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteNo",
              args: [999999, 1n * 10n ** 18n, 1],
            }),
            skipEstimation: true,
          })
        );

        expectEVMResult(block.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(block.result!.hash, context.ethers());
        expect(revertReason).toContain("NotOngoing");
      },
    });

    it({
      id: "T05",
      title: "should fail to vote with the wrong conviction",
      test: async function () {
        const block = await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteYes",
              args: [proposalIndex, 1n * 10n ** 18n, 7],
            }),
            skipEstimation: true,
          })
        );
        expectEVMResult(block.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(block.result!.hash, context.ethers());
        expect(revertReason).to.contain("Must be an integer between 0 and 6 included");
      },
    });
  },
});
