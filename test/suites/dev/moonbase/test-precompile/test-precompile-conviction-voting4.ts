import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  GLMR,
} from "@moonwall/util";
import { type Abi, decodeEventLog } from "viem";
import {
  ConvictionVoting,
  cancelProposal,
  createProposal,
  expectEVMResult,
  expectSubstrateEvent,
} from "../../../../helpers";

describeSuite({
  id: "D022820",
  title: "Precompiles - Conviction on General Admin Track",
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
      proposalIndex = await createProposal({ context, track: "generaladmin" });

      const block = await convictionVoting.voteYes(proposalIndex, GLMR, 1n).block();
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

        // general_admin is track 2
        const block = await convictionVoting
          .withPrivateKey(ETHAN_PRIVATE_KEY)
          .removeOtherVote(ALITH_ADDRESS, trackId, proposalIndex)
          .block();
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
        const block = await convictionVoting.removeVoteForTrack(proposalIndex, 2).block();
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
        // general_admin is track 2
        const block = await convictionVoting
          .withGas(2_000_000n)
          .removeVoteForTrack(proposalIndex, 0)
          .block();
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
        // general_admin is track 2
        const block = await convictionVoting
          .withPrivateKey(BALTATHAR_PRIVATE_KEY)
          .withGas(2_000_000n)
          .removeOtherVote(ALITH_ADDRESS, 2, proposalIndex)
          .block();
        expectEVMResult(block.result!.events, "Revert");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });
  },
});
