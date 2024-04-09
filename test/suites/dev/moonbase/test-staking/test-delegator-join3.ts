import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  charleth,
  ethan,
} from "@moonwall/util";

describeSuite({
  id: "D013445",
  title: "Staking - Delegator Join - wrong candidate delegation hint",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(charleth),
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
      ]);
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 1)
            .signAsync(ethan)
        );
        expect(block.result!.successful!).to.be.false;
        expect(block.result!.error!.name).to.equal("TooLowCandidateDelegationCountToDelegate");
      },
    });
  },
});
