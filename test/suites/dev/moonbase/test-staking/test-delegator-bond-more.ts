import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013440",
  title: "Staking - Bond More - no scheduled request",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(alith.address, MIN_GLMR_STAKING * 5n, 0, 0, 0, 0)
          .signAsync(ethan)
      );
    });

    it({
      id: "T01",
      title: "should succeed and increase total",
      test: async () => {
        const bondAmountBefore = (
          await context.polkadotJs().query.parachainStaking.delegatorState(ethan.address)
        ).unwrap().total;

        const increaseAmount = 5;
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegatorBondMore(alith.address, increaseAmount)
            .signAsync(ethan)
        );

        const bondAmountAfter = (
          await context.polkadotJs().query.parachainStaking.delegatorState(ethan.address)
        ).unwrap().total;
        expect(bondAmountAfter.eq(bondAmountBefore.addn(increaseAmount))).to.be.true;
      },
    });
  },
});
