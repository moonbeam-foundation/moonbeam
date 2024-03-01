import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013307",
  title: "Staking - Candidate Leave Schedule - valid request",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(ethan),
        { signer: alith, allowFailures: false }
      );
      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(ethan),
        { signer: alith, allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should change status to leaving at correct round",
      test: async () => {
        const candidatePool = (
          await context.polkadotJs().query.parachainStaking.candidatePool()
        ).map((c) => c.owner.toString());
        const candidateState = (
          await context.polkadotJs().query.parachainStaking.candidateInfo(ethan.address)
        ).unwrap();
        const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveCandidatesDelay;
        const currentRound = (await context.polkadotJs().query.parachainStaking.round()).current;

        expect(candidatePool).to.be.deep.equal([alith.address]);
        expect(candidateState.status.isLeaving).to.be.true;
        expect(candidateState.status.asLeaving.toNumber()).to.equal(
          currentRound.add(leaveDelay).toNumber()
        );
      },
    });
  },
});
