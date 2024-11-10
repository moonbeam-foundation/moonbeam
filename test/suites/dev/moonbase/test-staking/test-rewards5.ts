import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  ethan,
  dorothy,
  charleth,
  Percent,
} from "@moonwall/util";
import { jumpBlocks } from "../../../../helpers";
import { BN } from "@polkadot/util";

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
          .tx.sudo.sudo(
            context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig([
              {
                account: dorothy.address,
                percent: PBR_PERCENTAGE,
              },
              {
                account: charleth.address,
                percent: TREASURY_PERCENTAGE,
              },
            ])
          )
          .signAsync(alith),
      ]);
    });

    it({
      id: "T01",
      title: "Should act correctly upon inflation distribution config",
      test: async () => {
        const BLOCKS_PER_ROUND = 10;
        await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.sudo.sudo(
                context.polkadotJs().tx.parachainStaking.setBlocksPerRound(BLOCKS_PER_ROUND)
              )
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

        const currentHash = await context.polkadotJs().rpc.chain.getFinalizedHead();
        const currentBlockNumber = (
          await context.polkadotJs().rpc.chain.getHeader(currentHash)
        ).number.toNumber();
        const blocksToJump = BLOCKS_PER_ROUND - currentBlockNumber;
        console.log(`Jumping ${blocksToJump} blocks`);
        await jumpBlocks(context, blocksToJump);

        let pbrReward: bigint | undefined;
        let treasuryReward: bigint | undefined;

        (await context.polkadotJs().query.system.events()).forEach((event) => {
          if (context.polkadotJs().events.parachainStaking.InflationDistributed.is(event.event)) {
            if (event.event.data.account.toString() == dorothy.address) {
              pbrReward = event.event.data.value.toBigInt();
            } else if (event.event.data.account.toString() == charleth.address) {
              treasuryReward = event.event.data.value.toBigInt();
            }
          }
        });

        const payout = (
          await context.polkadotJs().query.parachainStaking.delayedPayouts(1)
        ).unwrap();
        const totalReward = payout.roundIssuance.toBigInt();
        const otherRewards = payout.totalStakingReward.toBigInt();

        expect(pbrReward).is.not.undefined;
        expect(treasuryReward).is.not.undefined;

        expect((pbrReward! + treasuryReward! - BigInt(1)).toString()).to.be.eq(
          new Percent(PBR_PERCENTAGE + TREASURY_PERCENTAGE)
            .of(new BN(totalReward.toString()))
            .toString()
        );

        expect((otherRewards + BigInt(1)).toString()).to.be.eq(
          new Percent(100 - PBR_PERCENTAGE - TREASURY_PERCENTAGE)
            .of(new BN(totalReward.toString()))
            .toString()
        );

        expect(pbrReward!.toString()).to.be.eq(
          new Percent(PBR_PERCENTAGE).of(new BN(totalReward.toString())).toString()
        );

        expect(treasuryReward!.toString()).to.be.eq(
          new Percent(TREASURY_PERCENTAGE).of(new BN(totalReward.toString())).toString()
        );
      },
    });
  },
});
