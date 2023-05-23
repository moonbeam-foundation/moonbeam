import "@moonbeam-network/api-augment";
import { DevModeContext, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, alith, baltathar } from "@moonwall/util";
import { verifyLatestBlockFees } from "../../../helpers/block.js";
import { calculate_vested_amount, getAccountPayable } from "../../../helpers/crowdloan.js";
import { RELAYCHAIN_ARBITRARY_ADDRESS_1, VESTING_PERIOD } from "../../../helpers/constants.js";

describeSuite({
  id: "D0709",
  title: "Crowdloan - Proxy claim",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const proxy = baltathar;

    it({
      id: "T01",
      title: "should be able to call crowdloan rewards with non-transfer proxy",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
              [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
            ])
          )
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();

        // Complete initialization
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.completeInitialization(
              initBlock.toBigInt() + VESTING_PERIOD
            )
          )
        );

        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();
        expect(isInitialized.isTrue).to.be.true;

        // GENESIS_ACCOUNT should be in accounts pauable
        const rewardInfo = await getAccountPayable(context, alith.address);
        expect(rewardInfo!.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
        expect(rewardInfo!.claimedReward.toBigInt()).to.equal(900_000n * GLMR);

        // CreateProxy
        await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(proxy.address, "NonTransfer", 0)
        );

        // three blocks elapsed
        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          3n
        );

        // Claim with proxy

        await context.createBlock(
          context.polkadotJs().tx.proxy
            .proxy(alith.address, null, context.polkadotJs().tx.crowdloanRewards.claim())
            .signAsync(proxy)
        );

        // Claimed amount should match
        const claimedRewards = (await getAccountPayable(context, alith.address))!.claimedReward;
        expect(claimedRewards.toBigInt()).to.equal(claimed);
      },
    });
  },
});