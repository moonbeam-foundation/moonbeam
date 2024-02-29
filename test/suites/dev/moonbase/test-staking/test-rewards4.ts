import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D013364",
  title: "Staking - Rewards - scheduled bond decrease request",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const EXTRA_BOND_AMOUNT = 1_000_000_000_000_000_000n;
    const BOND_AMOUNT = MIN_GLMR_STAKING + EXTRA_BOND_AMOUNT;

    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, BOND_AMOUNT, 0, 0)
            .signAsync(ethan),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, BOND_AMOUNT, 1, 0)
            .signAsync(baltathar),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, EXTRA_BOND_AMOUNT)
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should reward less than baltathar",
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

        const rewardedEthan = rewardedEvents.find(({ account }) => account == ethan.address);
        const rewardedBalathar = rewardedEvents.find(({ account }) => account == baltathar.address);
        expect(rewardedEthan).is.not.undefined;
        expect(rewardedBalathar).is.not.undefined;
        expect(
          rewardedBalathar!.amount,
          `Ethan's reward ${rewardedEthan!.amount} was not less than Balathar's \
      reward ${rewardedBalathar!.amount}`
        ).toBeGreaterThan(rewardedEthan!.amount);
      },
    });
  },
});
