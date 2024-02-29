import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, ETHAN_ADDRESS, ETHAN_PRIVATE_KEY } from "@moonwall/util";
import { Abi, decodeEventLog } from "viem";
import {
  expectEVMResult,
  extractRevertReason,
  expectSubstrateEvent,
  createProposal,
  ConvictionVoting,
} from "../../../../helpers";

describeSuite({
  id: "D012931",
  title: "Precompiles - Conviction Voting precompile",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVotingAbi: Abi;
    let convictionVoting: ConvictionVoting;

    beforeAll(async function () {
      const { abi } = fetchCompiledContract("ConvictionVoting");
      convictionVotingAbi = abi;
    });

    beforeEach(async function () {
      convictionVoting = new ConvictionVoting(context);
      proposalIndex = await createProposal({ context });
    });

    it({
      id: "T01",
      title: "should allow to vote yes for a proposal",
      test: async function () {
        const block = await convictionVoting.voteYes(proposalIndex, 1n * 10n ** 18n, 1n).block();

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
        const block = await convictionVoting.voteNo(proposalIndex, 1n * 10n ** 18n, 1n).block();

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
        const block1 = await convictionVoting.voteYes(proposalIndex, 1n * 10n ** 18n, 1n).block();
        expectEVMResult(block1.result!.events, "Succeed");

        const block2 = await convictionVoting.voteNo(proposalIndex, 1n * 10n ** 18n, 1n).block();
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
        const block = await convictionVoting
          .withGas(1_000_000n)
          .voteNo(999999, 1n * 10n ** 18n, 1n)
          .block();

        expectEVMResult(block.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(context, block.result!.hash);
        expect(revertReason).toContain("NotOngoing");
      },
    });

    it({
      id: "T05",
      title: "should fail to vote with the wrong conviction",
      test: async function () {
        const block = await convictionVoting
          .withGas(1_000_000n)
          .voteYes(proposalIndex, 1n * 10n ** 18n, 7n)
          .block();
        expectEVMResult(block.result!.events, "Revert", "Reverted");

        const revertReason = await extractRevertReason(context, block.result!.hash);
        expect(revertReason).to.contain("Must be an integer between 0 and 6 included");
      },
    });

    it({
      id: "T06",
      title: "should allow to vote split",
      test: async function () {
        const ayes = 1n * 10n ** 18n;
        const nays = 2n * 10n ** 18n;
        // Vote split
        const block = await convictionVoting.voteSplit(proposalIndex, ayes, nays).block();

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
      id: "T07",
      title: "should allow to vote split with abstain",
      test: async function () {
        const ayes = 1n * 10n ** 18n;
        const nays = 2n * 10n ** 18n;
        const abstain = 3n * 10n ** 18n;
        // Vote split
        const block = await convictionVoting
          .voteSplitAbstain(proposalIndex, ayes, nays, abstain)
          .block();

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
      id: "T08",
      title: "should allow to delegate a vote",
      test: async function () {
        const trackId = 0;
        const amount = 1n * 10n ** 10n;
        const conviction = 1;
        // Delegates the vote
        const block = await convictionVoting
          .withPrivateKey(ETHAN_PRIVATE_KEY)
          .delegate(trackId, ALITH_ADDRESS, conviction, amount)
          .block();

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
        const {
          data: [who, target],
        } = expectSubstrateEvent(block, "convictionVoting", "Delegated");
        expect(who.toString()).to.equal(ETHAN_ADDRESS);
        expect(target.toString()).to.equal(ALITH_ADDRESS);
        // Undelegates the vote
        {
          const block = await convictionVoting
            .withPrivateKey(ETHAN_PRIVATE_KEY)
            .undelegate(trackId)
            .block();

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
          const {
            data: [who],
          } = expectSubstrateEvent(block, "convictionVoting", "Undelegated");
          expect(who.toString()).to.equal(ETHAN_ADDRESS);
        }
      },
    });
  },
});
