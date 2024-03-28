import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { Abi, decodeEventLog } from "viem";
import {
  ConvictionVoting,
  createProposal,
  expectEVMResult,
  expectSubstrateEvent,
} from "../../../../helpers";

describeSuite({
  id: "D012833",
  title: "Precompiles - Conviction on Root Track",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVotingAbi: Abi;
    let convictionVoting: ConvictionVoting;

    beforeAll(async function () {
      const { abi } = fetchCompiledContract("ConvictionVoting");
      convictionVoting = new ConvictionVoting(context);
      convictionVotingAbi = abi;
    });

    beforeEach(async function () {
      proposalIndex = await createProposal({ context });

      const block = await convictionVoting.voteYes(proposalIndex, 1n * 10n ** 18n, 1n).block();
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
        const block = await convictionVoting.removeVote(proposalIndex).block();
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
        const trackId = 0;

        const block = await convictionVoting.removeVoteForTrack(proposalIndex, trackId).block();
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

        // Verifies the Subsrtate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
      },
    });
  },
});
