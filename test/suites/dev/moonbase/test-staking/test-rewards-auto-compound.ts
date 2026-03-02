import "@moonbeam-network/api-augment";
import { MIN_GLMR_DELEGATOR, alith, beforeAll, describeSuite, ethan, expect } from "moonwall";
import { jumpRounds, getRewardedAndCompoundedEvents } from "../../../../helpers";

describeSuite({
  id: "D023352",
  title: "Staking - Rewards Auto-Compound - no auto-compound config",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              0,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should not compound reward and emit no event",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const events = await getRewardedAndCompoundedEvents(context, blockHash);
        const rewardedEvent = events.rewarded.find(({ account }: any) => account === ethan.address);
        const compoundedEvent = events.compounded.find(
          ({ delegator }: any) => delegator === ethan.address
        );

        expect(rewardedEvent, "delegator was not rewarded").to.not.be.undefined;
        expect(compoundedEvent, "delegator reward was erroneously compounded").to.be.undefined;
      },
    });
  },
});
