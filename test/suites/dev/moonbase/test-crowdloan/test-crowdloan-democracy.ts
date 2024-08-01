import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  fastFowardToNextEvent,
  maximizeConvictionVotingOf,
  whiteListTrackNoSend,
} from "@moonwall/cli";
import { DEFAULT_GENESIS_BALANCE, ethan, GLMR, GOLIATH_ADDRESS } from "@moonwall/util";
import { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import {
  getAccountPayable,
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  RELAYCHAIN_ARBITRARY_ADDRESS_2,
  VESTING_PERIOD,
} from "../../../../helpers";

describeSuite({
  id: "D010805",
  title: "Crowdloan - Democracy",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to initialize through democracy",
      test: async () => {
        const batchedCalls: SubmittableExtrinsic[] = [];
        batchedCalls.push(
          context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
            [RELAYCHAIN_ARBITRARY_ADDRESS_1, GOLIATH_ADDRESS, 1_500_000n * GLMR],
            [RELAYCHAIN_ARBITRARY_ADDRESS_2, null, 1_500_000n * GLMR],
          ])
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        batchedCalls.push(
          context
            .polkadotJs()
            .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
        );

        // Here we build the utility call
        const proposal = context.polkadotJs().tx.utility.batchAll(batchedCalls);

        await whiteListTrackNoSend(context, proposal);

        await maximizeConvictionVotingOf(context, [ethan], 0);
        await context.createBlock();

        await fastFowardToNextEvent(context); // ⏩️ until preparation done
        await fastFowardToNextEvent(context); // ⏩️ until proposal confirmed
        await fastFowardToNextEvent(context); // ⏩️ until proposal enacted

        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();

        expect(isInitialized.toHuman()).to.be.true;

        const reward_info_associated = await getAccountPayable(context, GOLIATH_ADDRESS);

        const reward_info_unassociated = (
          await context
            .polkadotJs()
            .query.crowdloanRewards.unassociatedContributions(RELAYCHAIN_ARBITRARY_ADDRESS_2)
        ).unwrap();

        // Check payments
        expect(reward_info_associated!.totalReward.toBigInt()).toBe(1_500_000n * GLMR);
        expect(reward_info_associated!.claimedReward.toBigInt()).toBe(450_000n * GLMR);
        expect(reward_info_unassociated.totalReward.toBigInt()).toBe(1_500_000n * GLMR);
        expect(reward_info_unassociated.claimedReward.toBigInt()).toBe(0n);

        // check balances
        const account = await context.polkadotJs().query.system.account(GOLIATH_ADDRESS);
        expect(account.data.free.toBigInt() - DEFAULT_GENESIS_BALANCE).toBe(
          reward_info_associated!.claimedReward.toBigInt()
        );
      },
    });
  },
});
