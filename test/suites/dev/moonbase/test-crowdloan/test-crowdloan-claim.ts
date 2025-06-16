import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ALITH_GENESIS_FREE_BALANCE, GLMR, alith } from "@moonwall/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  VESTING_PERIOD,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  calculate_vested_amount,
  getAccountPayable,
} from "../../../../helpers";

describeSuite({
  id: "D020703",
  title: "Crowdloan - make claim",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to make a first claim",
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

        const rewardInfo = await getAccountPayable(context, ALITH_ADDRESS);
        const claimed = await calculate_vested_amount(
          rewardInfo!.totalReward.toBigInt(),
          rewardInfo!.claimedReward.toBigInt(),
          2n
        );

        const transfer = context.polkadotJs().tx.crowdloanRewards.claim();
        await transfer.signAndSend(alith);
        const details = await context.polkadotJs().rpc.payment.queryFeeDetails(transfer.toHex());
        const claimFee =
          details.inclusionFee.unwrap().baseFee.toBigInt() +
          details.inclusionFee.unwrap().lenFee.toBigInt() +
          details.inclusionFee.unwrap().adjustedWeightFee.toBigInt();

        await context.createBlock();
        expect(
          (await getAccountPayable(context, ALITH_ADDRESS))!.claimedReward.toBigInt()
        ).to.equal(claimed);

        expect(
          (await context.viem().getBalance({ address: ALITH_ADDRESS })) -
            ALITH_GENESIS_TRANSFERABLE_BALANCE
        ).toBe(claimed - claimFee); // reduce the claim fee part;
        const account = await context.polkadotJs().query.system.account(ALITH_ADDRESS);
        expect(account.data.free.toBigInt() - ALITH_GENESIS_FREE_BALANCE).toBe(claimed - claimFee);
      },
    });
  },
});
