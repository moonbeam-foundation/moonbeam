import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, alith } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  VESTING_PERIOD,
  calculate_vested_amount,
  getAccountPayable,
} from "../../../../helpers";

describeSuite({
  id: "D020712",
  title: "Crowdloan - small amount",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should be able to register the genesis account - with small amount",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.initializeRewardVec([
                  [RELAYCHAIN_ARBITRARY_ADDRESS_1, ALITH_ADDRESS, 3_000_000n * GLMR],
                ])
            )
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
            )
        );

        expect((await getAccountPayable(context, ALITH_ADDRESS))!.totalReward.toBigInt()).to.equal(
          3_000_000n * GLMR
        );

        const rewardInfo = await getAccountPayable(context, ALITH_ADDRESS);
        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          2n
        );

        await context.polkadotJs().tx.crowdloanRewards.claim().signAndSend(alith);
        await context.createBlock();

        const isPayable4 = await getAccountPayable(context, ALITH_ADDRESS);
        expect(isPayable4!.claimedReward.toBigInt()).to.equal(claimed);
      },
    });
  },
});
