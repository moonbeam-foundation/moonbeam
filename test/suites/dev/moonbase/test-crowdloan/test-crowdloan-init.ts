import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, alith, baltathar } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  VESTING_PERIOD,
  getAccountPayable,
  verifyLatestBlockFees,
} from "../../../../helpers";

describeSuite({
  id: "D020708",
  title: "Crowdloan - Init",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should check initial state",
      test: async function () {
        const isPayable = await getAccountPayable(context, ALITH_ADDRESS);
        expect(isPayable, "Genesis is not registered").to.equal(null);
      },
    });

    it({
      id: "T02",
      title: "should be able to register the genesis account for reward",
      test: async function () {
        // should be
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

        await verifyLatestBlockFees(context, 3_000_000n);

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

        expect(
          (await getAccountPayable(context, ALITH_ADDRESS))!.totalReward.toBigInt(),
          "Unable to register the genesis account for reward"
        ).to.equal(3_000_000n * GLMR);
        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();
        expect(isInitialized.toHuman()).to.be.true;
      },
    });

    it({
      id: "T03",
      title: "should not be able to call initializeRewardVec another time",
      test: async function () {
        await context
          .polkadotJs()
          .tx.sudo.sudo(
            context
              .polkadotJs()
              .tx.crowdloanRewards.initializeRewardVec([
                [RELAYCHAIN_ARBITRARY_ADDRESS_1, baltathar.address, 1000n * GLMR],
              ])
          )
          .signAndSend(alith);
        await context.createBlock();
        expect(await getAccountPayable(context, baltathar.address)).to.equal(null);
      },
    });
  },
});
