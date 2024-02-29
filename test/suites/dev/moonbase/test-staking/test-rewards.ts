import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D013361",
  title: "Staking - Rewards - no scheduled requests",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_STAKING, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should reward full amount",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        const blockHash = (await context.createBlock()).block.hash.toString();
        const allEvents = await (await context.polkadotJs().at(blockHash)).query.system.events();
        const rewardedEvents = allEvents.reduce(
          (acc: { account: string; amount: bigint }[], event) => {
            if (context.polkadotJs().events.parachainStaking.Rewarded.is(event.event)) {
              acc.push({
                account: event.event.data.account.toString(),
                amount: event.event.data.rewards.toBigInt(),
              });
            }
            return acc;
          },
          []
        );

        expect(
          rewardedEvents.some(({ account }) => account == ethan.address),
          "delegator was not rewarded"
        ).to.be.true;
      },
    });
  },
});
