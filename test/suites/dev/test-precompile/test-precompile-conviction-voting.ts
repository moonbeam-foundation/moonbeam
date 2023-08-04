import "@moonbeam-network/api-augment";
import {
  DevModeContext,
  beforeAll,
  beforeEach,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  PRECOMPILE_CONVICTION_VOTING_ADDRESS,
  createViemTransaction,
} from "@moonwall/util";
import { Abi, decodeEventLog, encodeFunctionData } from "viem";
import { expectEVMResult, extractRevertReason } from "../../../helpers/eth-transactions.js";
import { expectSubstrateEvent } from "../../../helpers/expect.js";
import { cancelProposal, createProposal } from "../../../helpers/voting.js";

async function voteYes(
  context: DevModeContext,
  convictionVotingAbi: Abi,
  proposalIndex: number,
  amount: bigint,
  conviction: number
) {
  const rawTx = await createViemTransaction(context, {
    to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
    data: encodeFunctionData({
      abi: convictionVotingAbi,
      functionName: "voteYes",
      args: [proposalIndex, amount, conviction],
    }),
  });
  const block = await context.createBlock(rawTx);
  return block;
}

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
        const block = await voteYes(
          context,
          convictionVotingAbi,
          proposalIndex,
          1n * 10n ** 18n,
          1
        );

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
        const block1 = await voteYes(
          context,
          convictionVotingAbi,
          proposalIndex,
          1n * 10n ** 18n,
          1
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
        const revertReason = await extractRevertReason(context, block.result!.hash);
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
        const revertReason = await extractRevertReason(context, block.result!.hash);
        expect(revertReason).to.contain("Must be an integer between 0 and 6 included");
      },
    });

    it({
      id: "T06",
      title: "should allow to remove a vote",
      test: async function () {
        // Vote Yes
        let tx = await createViemTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "voteYes",
            args: [proposalIndex, 1n * 10n ** 18n, 1],
          }),
        });
        await context.createBlock(tx);

        // Remove vote
        const rawTx = await createViemTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "removeVote",
            args: [proposalIndex],
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

        expect(evmLog.eventName, "Wrong event").to.equal("VoteRemoved");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);

        // Verifies the Substrate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T07",
      title: "should allow to vote split",
      test: async function () {
        const ayes = 1n * 10n ** 18n;
        const nays = 2n * 10n ** 18n;
        // Vote split
        const rawTx = await createViemTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "voteSplit",
            args: [proposalIndex, ayes, nays],
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

        expect(evmLog.eventName, "Wrong event").to.equal("VoteSplit");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.aye).to.equal(ayes);
        expect(evmLog.args.nay).to.equal(nays);

        // Verifies the Substrate side
        // Since the vote is split, the total amount of votes is equal to %10
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(ayes / 10n);
        expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(nays / 10n);
      },
    });

    it({
      id: "T08",
      title: "should allow to vote split with abstain",
      test: async function () {
        const ayes = 1n * 10n ** 18n;
        const nays = 2n * 10n ** 18n;
        const abstain = 3n * 10n ** 18n;
        // Vote split
        const rawTx = await createViemTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "voteSplitAbstain",
            args: [proposalIndex, ayes, nays, abstain],
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

        expect(evmLog.eventName, "Wrong event").to.equal("VoteSplitAbstained");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.aye).to.equal(ayes);
        expect(evmLog.args.nay).to.equal(nays);
        expect(evmLog.args.abstain).to.equal(abstain);

        // Verifies the Substrate side
        // Since the vote is split, the total amount of votes is equal to %10
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(ayes / 10n);
        expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(nays / 10n);
      },
    });

    it({
      id: "T09",
      title: "should allow to remove a vote for a track",
      test: async function () {
        await voteYes(context, convictionVotingAbi, proposalIndex, 1n * 10n ** 18n, 1);

        const trackId = 0;
        // Removes the vote for the root track
        const rawTx = await createViemTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "removeVoteForTrack",
            args: [proposalIndex, trackId],
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

        expect(evmLog.eventName, "Wrong event").to.equal("VoteRemovedForTrack");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.trackId).to.equal(trackId);

        // Verifies the Substrate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T10",
      title: "should allow to remove a vote from another address",
      test: async function () {
        // Alith votes yes
        await voteYes(context, convictionVotingAbi, proposalIndex, 1n * 10n ** 18n, 1);
        // Cancel the proposal
        await cancelProposal(context, proposalIndex);

        // Ethan emoves the vote by Alith
        const trackId = 0n;
        const rawTx = await createViemTransaction(context, {
          privateKey: ETHAN_PRIVATE_KEY,
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "removeOtherVote",
            args: [ALITH_ADDRESS, trackId, proposalIndex],
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

        expect(evmLog.eventName, "Wrong event").to.equal("VoteRemovedOther");
        expect(evmLog.args.caller).to.equal(ETHAN_ADDRESS);
        expect(evmLog.args.target).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.trackId).to.equal(0);

        // Verifies the Substrate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().isCancelled).to.equal(true);
      },
    });

    it({
      id: "T11",
      title: "should allow to delegate a vote",
      test: async function () {
        const trackId = 0;
        const amount = 1n * 10n ** 10n;
        const conviction = 1;
        // Delegates the vote
        const rawTx = await createViemTransaction(context, {
          privateKey: ETHAN_PRIVATE_KEY,
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "delegate",
            args: [trackId, ALITH_ADDRESS, conviction, amount],
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

        expect(evmLog.eventName, "Wrong event").to.equal("Delegated");
        expect(evmLog.args.trackId).to.equal(trackId);
        expect(evmLog.args.from).to.equal(ETHAN_ADDRESS);
        expect(evmLog.args.to).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.delegatedAmount).to.equal(amount);
        expect(evmLog.args.conviction).to.equal(conviction);

        // Verifies the Substrate side
        expectSubstrateEvent(block, "convictionVoting", "Delegated");

        // Undelegates the vote
        {
          const rawTx = await createViemTransaction(context, {
            privateKey: ETHAN_PRIVATE_KEY,
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "undelegate",
              args: [trackId],
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

          expect(evmLog.eventName, "Wrong event").to.equal("Undelegated");
          expect(evmLog.args.trackId).to.equal(trackId);
          expect(evmLog.args.caller).to.equal(ETHAN_ADDRESS);

          // Verifies the Substrate side
          expectSubstrateEvent(block, "convictionVoting", "Undelegated");
        }
      },
    });
  },
});
