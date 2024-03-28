import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013402",
  title: "Staking - Candidate Join - bond less than min",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const minCandidateStk = context.polkadotJs().consts.parachainStaking.minCandidateStk;
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(minCandidateStk.subn(10), 1)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("CandidateBondBelowMin");
      },
    });

    it({
      id: "T02",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(alith)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("CandidateExists");
      },
    });

    it({
      id: "T03",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 0)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("TooLowCandidateCountWeightHintJoinCandidates");
      },
    });
  },
});
