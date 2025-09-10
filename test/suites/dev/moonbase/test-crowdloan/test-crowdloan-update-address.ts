import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, generateKeyringPair } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  VESTING_PERIOD,
  calculate_vested_amount,
  getAccountPayable,
} from "../../../../helpers";

describeSuite({
  id: "D020714",
  title: "Crowdloan - Update Address",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const toUpdateAccount = generateKeyringPair();

    it({
      id: "T01",
      title: "should be able to update reward address",
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

        // Complete initialization
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
            )
        );

        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();

        expect(isInitialized.isTrue).to.be.true;

        // GENESIS_ACCOUNT should be in accounts pauable
        const rewardInfo = await getAccountPayable(context, ALITH_ADDRESS);
        expect(rewardInfo!.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
        expect(rewardInfo!.claimedReward.toBigInt()).to.equal(900_000n * GLMR);

        // three blocks elapsed
        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          2n
        );

        await context.createBlock(context.polkadotJs().tx.crowdloanRewards.claim());

        // Claimed amount should match
        const claimedRewards = (await getAccountPayable(context, ALITH_ADDRESS))!.claimedReward;
        expect(claimedRewards.toBigInt()).to.equal(claimed);

        // Let's update the reward address
        await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.updateRewardAddress(toUpdateAccount.address)
        );

        // GENESIS_ACCOUNT should no longer be in accounts payable
        expect(await getAccountPayable(context, ALITH_ADDRESS)).to.be.null;

        // toUpdateAccount should be in accounts paYable
        const updateRewardInfo = await getAccountPayable(context, toUpdateAccount.address);
        expect(updateRewardInfo!.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
        expect(updateRewardInfo!.claimedReward.toBigInt()).to.equal(claimed);
      },
    });
  },
});
