import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  RELAYCHAIN_ARBITRARY_ADDRESS_2,
  VESTING_PERIOD,
} from "../../../../helpers/constants.js";

describeSuite({
  id: "D020706",
  title: "Crowdloan",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should be able to burn the dust",
      test: async function () {
        await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
              [RELAYCHAIN_ARBITRARY_ADDRESS_1, ALITH_ADDRESS, 1_500_000n * GLMR],
              [RELAYCHAIN_ARBITRARY_ADDRESS_2, null, 1_499_999_999_999_999_999_999_999n],
            ])
          )
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        const previousIssuance = await context.polkadotJs().query.balances.totalIssuance();

        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
            )
        );

        const issuance = await context.polkadotJs().query.balances.totalIssuance();
        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();
        expect(isInitialized.isTrue).to.be.true;
        expect(issuance.toBigInt()).to.eq(previousIssuance.toBigInt() - 1n);
      },
    });
  },
});
