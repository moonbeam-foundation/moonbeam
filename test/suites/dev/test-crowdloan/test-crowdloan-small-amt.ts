import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { RELAYCHAIN_ARBITRARY_ADDRESS_1, VESTING_PERIOD } from "../../../helpers/constants.js";
import { calculate_vested_amount, getAccountPayable } from "../../../helpers/crowdloan.js";

describeSuite({
  id: "D0712",
  title: "Crowdloan - small amount",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should be able to register the genesis account - with small amount",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
              [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
            ])
          )
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.completeInitialization(
              initBlock.toBigInt() + VESTING_PERIOD
            )
          )
        );

        expect((await getAccountPayable(context, alith.address))!.totalReward.toBigInt()).to.equal(
          3_000_000n * GLMR
        );

        const rewardInfo = await getAccountPayable(context, alith.address);
        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          2n
        );

        await context.polkadotJs().tx.crowdloanRewards.claim().signAndSend(alith);
        await context.createBlock();

        const isPayable4 = await getAccountPayable(context, alith.address);
        expect(isPayable4!.claimedReward.toBigInt()).to.equal(claimed);
      },
    });
  },
});
