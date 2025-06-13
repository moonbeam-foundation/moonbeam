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
  id: "D020702",
  title: "Crowdloan - claim updates balances",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should show me the money after 5 blocks, after first claim was called",
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

        const rewardInfo = await getAccountPayable(context, ALITH_ADDRESS);
        await context.polkadotJs().tx.crowdloanRewards.claim().signAndSend(alith);
        await context.createBlock();
        await context.createBlock();
        await context.createBlock();

        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          5n
        );

        await context.polkadotJs().tx.crowdloanRewards.claim().signAndSend(alith);
        await context.createBlock();
        const isPayable4 = await getAccountPayable(context, ALITH_ADDRESS);
        expect(isPayable4!.claimedReward.toBigInt()).to.equal(claimed);
      },
    });
  },
});
