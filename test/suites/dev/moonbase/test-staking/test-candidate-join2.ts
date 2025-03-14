import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013403",
  title: "Staking - Candidate Join - already a delegator",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 0, 0, 0, 0)
          .signAsync(ethan),
        { signer: alith, allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("DelegatorExists");
      },
    });
  },
});
