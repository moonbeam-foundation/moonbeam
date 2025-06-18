import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, baltathar } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  VESTING_PERIOD,
  getAccountPayable,
} from "../../../../helpers";

describeSuite({
  id: "D020701",
  title: "Crowdloan - Proxy transfer",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const proxy = baltathar;

    it({
      id: "T01",
      title: "should NOT be able to call non-claim extrinsic with non-transfer proxy",
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

        // CreateProxy
        await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(proxy.address, "NonTransfer", 0)
        );

        // Should not be able to do this
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              ALITH_ADDRESS,
              null,
              context.polkadotJs().tx.crowdloanRewards.updateRewardAddress(proxy.address)
            )
            .signAsync(proxy)
        );
        expect(result?.events[1].event.method).to.eq("ProxyExecuted");
        expect(result?.events[1].event.data[0].toString()).to.be.eq(
          `{"err":{"module":{"index":0,"error":"0x05000000"}}}`
        );

        // Genesis account still has the money
        const updatedRewardInfo = await getAccountPayable(context, ALITH_ADDRESS);
        expect(updatedRewardInfo!.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
        expect(updatedRewardInfo!.claimedReward.toBigInt()).to.equal(900_000n * GLMR);
      },
    });
  },
});
