import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013341",
  title: "Staking - Bond More - bond less scheduled",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_STAKING * 5n, 0, 0)
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, 10n)
          .signAsync(ethan),
        { allowFailures: false }
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
