import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, Percent, alith, ethan } from "@moonwall/util";
import { jumpRounds, getRewardedAndCompoundedEvents } from "../../../../helpers";

describeSuite({
  id: "D013354",
  title: "Staking - Rewards Auto-Compound - 1% auto-compound",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            1,
            0,
            0,
            0
          )
          .signAsync(ethan),
      ]);
    });

    it({
      id: "T01",
      title: "should compound 1% reward",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(
          ({ account }: any) => account === ethan.address
        ) as any;
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        ) as any;

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(
          compoundedEvent!.amount.toString(),
          "delegator did not get 1% of their rewarded auto-compounded"
        ).to.equal(new Percent(1).ofCeil(rewardedEvent!.amount).toString());
      },
    });
  },
});
