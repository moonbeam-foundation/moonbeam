import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_STAKING, alith, baltathar, ethan, dorothy, charleth } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D0134655",
  title: "Staking - Rewards - Bond + Treasury",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const BOND_AMOUNT = MIN_GLMR_STAKING + 1_000_000_000_000_000_000n;
    const PBR_PERCENTAGE = 10;
    const TREASURY_PERCENTAGE = 20;

    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig([
            {
              account: dorothy.address,
              percent: PBR_PERCENTAGE,
            },
            {
              account: charleth.address,
              percent: TREASURY_PERCENTAGE,
            }
          ]))
          .signAsync(alith),
      ]);

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

    });

    it({
      id: "T01",
      title: "should reward charleth and dorothy correct amounts",
      test: async () => {
        const rewardDelay = context.polkadotJs().consts.parachainStaking.rewardPaymentDelay;
        let blockHash = (await context.createBlock()).block.hash.toString();
        const allEvents = await (await context.polkadotJs().at(blockHash)).query.system.events();
        await jumpRounds(context, rewardDelay.addn(1).toNumber());
        blockHash = (await context.createBlock()).block.hash.toString();
        allEvents.push(...await (await context.polkadotJs().at(blockHash)).query.system.events());

        const rewardedEvents = allEvents.reduce(
          (acc: { account: string; amount: bigint }[], event) => {
            console.log(event.event.section, event.event.method);
            if (context.polkadotJs().events.parachainStaking.Rewarded.is(event.event)) {
              acc.push({
                account: event.event.data.account.toString(),
                amount: event.event.data.rewards.toBigInt(),
              });
            } else if (context.polkadotJs().events.parachainStaking.InflationDistributed.is(event.event)) {
              acc.push({
                account: event.event.data.account.toString(),
                amount: event.event.data.value.toBigInt(),
              });
            }
            return acc;
          },
          []
        );

        const rewardedEthan = rewardedEvents.find(({ account }) => account == ethan.address);
        const rewardedBalathar = rewardedEvents.find(({ account }) => account == baltathar.address);

        const rewardedPbr = rewardedEvents.find(({ account }) => account == dorothy.address);
        const rewardedTreasury = rewardedEvents.find(({ account }) => account == charleth.address);



        expect(rewardedEthan).is.not.undefined;
        expect(rewardedBalathar).is.not.undefined;
        expect(rewardedPbr).is.not.undefined;
        expect(rewardedTreasury).is.not.undefined;

        const totalReward = rewardedEvents.reduce((acc, { amount }) => acc + amount, 0n);
        const reservedReward = rewardedPbr!.amount + rewardedTreasury!.amount;
        const otherReward = totalReward - reservedReward;
        const otherPercentage = BigInt(100 - PBR_PERCENTAGE - TREASURY_PERCENTAGE);

        const reservedRewardPercentage = ((reservedReward * 100n) / totalReward);
        const actualOtherPercentage = ((otherReward * 100n) / totalReward);

        expect(reservedRewardPercentage.toString(), "Reserved reward percentage is not correct")
          .toEqual((PBR_PERCENTAGE + TREASURY_PERCENTAGE).toString());
        expect(actualOtherPercentage.toString(), "Other reward percentage is not correct")
          .toEqual(otherPercentage.toString());

        const pbrPercentage = (rewardedPbr!.amount * 100n) / totalReward;
        const treasuryPercentage = (rewardedTreasury!.amount * 100n) / totalReward;

        expect(pbrPercentage.toString(), "PBR reward percentage is not correct")
          .toEqual(PBR_PERCENTAGE.toString());
        expect(treasuryPercentage.toString(), "Treasury reward percentage is not correct")
          .toEqual(TREASURY_PERCENTAGE.toString());
      },
    });
  },
});
