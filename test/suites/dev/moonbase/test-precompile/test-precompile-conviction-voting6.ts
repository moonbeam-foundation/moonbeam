import "@moonbeam-network/api-augment";
import { beforeAll, beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, BALTATHAR_PRIVATE_KEY, GLMR } from "@moonwall/util";
import { expectEVMResult, createProposal, ConvictionVoting } from "../../../../helpers";

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
// Be careful to not reach the maximum number of proposals.
describeSuite({
  id: "D012936",
  title: "Precompiles - ClassLocksFor & VotingFor",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVoting: ConvictionVoting;

    beforeAll(async function () {
      proposalIndex = await createProposal({ context });

      convictionVoting = new ConvictionVoting(context);
      const blockAlith_1 = await convictionVoting.voteYes(proposalIndex, GLMR, 1n).block();
      expectEVMResult(blockAlith_1.result!.events, "Succeed");

      const blockAlith_2 = await convictionVoting.voteYes(proposalIndex, 2n * GLMR, 2n).block();
      expectEVMResult(blockAlith_2.result!.events, "Succeed");

      const blockBaltathar = await convictionVoting
        .withPrivateKey(BALTATHAR_PRIVATE_KEY)
        .voteYes(proposalIndex, 3n * GLMR, 3n)
        .block();
      expectEVMResult(blockBaltathar.result!.events, "Succeed");
    });

    beforeEach(async function () {
      convictionVoting = new ConvictionVoting(context);
    });

    it({
      id: "T01",
      title: "should return classLocksFor alith",
      test: async function () {
        const result = (await convictionVoting.classLocksFor(ALITH_ADDRESS).tx()) as any;

        expect(result.length).to.equal(1);
        expect(result[0].trackId).to.equal(0);
        expect(result[0].amount).to.equal(2n * 10n ** 18n);
      },
    });

    it({
      id: "T02",
      title: "should return classLocksFor baltathar",
      test: async function () {
        const result = (await convictionVoting.classLocksFor(BALTATHAR_ADDRESS).tx()) as any;

        expect(result.length).to.equal(1);
        expect(result[0].trackId).to.equal(0);
        expect(result[0].amount).to.equal(3n * 10n ** 18n);
      },
    });

    it({
      id: "T03",
      title: "should return votingFor alith",
      test: async function () {
        const result = (await convictionVoting.votingFor(ALITH_ADDRESS, proposalIndex).tx()) as any;

        expect(result.casting.votes).to.have.lengthOf(1);
        expect(result.casting.votes[0].pollIndex).to.equal(0);
        expect(result.casting.votes[0].accountVote.isStandard).to.be.true;
        expect(result.casting.votes[0].accountVote.isSplit).to.be.false;
        expect(result.casting.votes[0].accountVote.isSplitAbstain).to.be.false;
        expect(result.casting.votes[0].accountVote.standard.vote.aye).to.be.true;
        expect(result.casting.votes[0].accountVote.standard.vote.conviction).to.equal(2);
        expect(result.casting.votes[0].accountVote.standard.balance).to.equal(2n * 10n ** 18n);
        expect(result.casting.prior.balance).to.equal(0n);
        expect(result.casting.delegations.votes).to.equal(0n);
        expect(result.casting.delegations.capital).to.equal(0n);
        expect(result.isCasting).to.be.true;
        expect(result.isDelegating).to.be.false;
      },
    });

    it({
      id: "T04",
      title: "should return votingFor baltathar",
      test: async function () {
        const result = (await convictionVoting
          .votingFor(BALTATHAR_ADDRESS, proposalIndex)
          .tx()) as any;
        expect(result.casting.votes).to.have.lengthOf(1);
        expect(result.casting.votes[0].pollIndex).to.equal(0);
        expect(result.casting.votes[0].accountVote.isStandard).to.be.true;
        expect(result.casting.votes[0].accountVote.isSplit).to.be.false;
        expect(result.casting.votes[0].accountVote.isSplitAbstain).to.be.false;
        expect(result.casting.votes[0].accountVote.standard.vote.aye).to.be.true;
        expect(result.casting.votes[0].accountVote.standard.vote.conviction).to.equal(3);
        expect(result.casting.votes[0].accountVote.standard.balance).to.equal(3n * 10n ** 18n);
        expect(result.casting.prior.balance).to.equal(0n);
        expect(result.casting.delegations.votes).to.equal(0n);
        expect(result.casting.delegations.capital).to.equal(0n);
        expect(result.isCasting).to.be.true;
        expect(result.isDelegating).to.be.false;
      },
    });
  },
});
