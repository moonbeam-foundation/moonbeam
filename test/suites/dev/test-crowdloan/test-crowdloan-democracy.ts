import "@moonbeam-network/api-augment";
import { describeSuite, expect, instantFastTrack } from "@moonwall/cli";
import { DEFAULT_GENESIS_BALANCE, GLMR, GOLIATH_ADDRESS, VOTE_AMOUNT } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  RELAYCHAIN_ARBITRARY_ADDRESS_2,
  VESTING_PERIOD,
} from "../../../helpers/constants.js";
import { getAccountPayable } from "../../../helpers/crowdloan.js";

describeSuite({
  id: "D0705",
  title: "Crowdloan - Democracy",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to initialize through democracy",
      test: async function () {
        const calls = [];
        // We are gonna put the initialization and completion in a batch_all utility call
        calls.push(
          context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
            [RELAYCHAIN_ARBITRARY_ADDRESS_1, GOLIATH_ADDRESS, 1_500_000n * GLMR],
            [RELAYCHAIN_ARBITRARY_ADDRESS_2, null, 1_500_000n * GLMR],
          ])
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        calls.push(
          context
            .polkadotJs()
            .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
        );

        // Here we build the utility call
        const proposal = context.polkadotJs().tx.utility.batchAll(calls);

        await instantFastTrack(context, proposal);

        // vote
        await context.createBlock(
          context.polkadotJs().tx.democracy.vote(0, {
            Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
          })
        );

        // referendumInfoOf
        const referendumInfoOf = (
          await context.polkadotJs().query.democracy.referendumInfoOf(0)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        const blockNumber = (await context.polkadotJs().rpc.chain.getHeader()).number.toNumber();
        for (let i = 0; i < onGoing.end.toNumber() - blockNumber + 1; i++) {
          await context.createBlock();
        }

        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();

        expect(isInitialized.toHuman()).to.be.true;

        // Get reward info of associated
        const reward_info_associated = await getAccountPayable(context, GOLIATH_ADDRESS);

        // Get reward info of unassociated
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
