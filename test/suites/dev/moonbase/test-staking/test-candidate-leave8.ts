import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013412",
  title: "Staking - Candidate Leave Cancel - leave scheduled",
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
      title: "should succeed",
      test: async () => {
        const candidateStateBefore = (
          await context.polkadotJs().query.parachainStaking.candidateInfo(ethan.address)
        ).unwrap();
        expect(candidateStateBefore.status.isLeaving).to.be.true;

        const block = await context.createBlock(
          context.polkadotJs().tx.parachainStaking.cancelLeaveCandidates(2).signAsync(ethan)
        );
        expect(block.result!.successful).to.be.true;

        const candidateStateAfter = (
          await context.polkadotJs().query.parachainStaking.candidateInfo(ethan.address)
        ).unwrap();
        expect(candidateStateAfter.status.isActive).to.be.true;
      },
    });
  },
});
