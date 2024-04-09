import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D013437",
  title: "Staking - Delegation Scheduled Requests - cancel scheduled bond less",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const LESS_AMOUNT = 10n;

    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR + LESS_AMOUNT, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
          .signAsync(ethan)
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();
        const delegationRequests = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();

        expect(delegationRequests[0].delegator.toString()).toBe(ethan.address);
        expect(delegationRequests[0].whenExecutable.toNumber()).toBe(currentRound + roundDelay);
        expect(delegationRequests[0].action.isDecrease).toBe(true);
        expect(delegationRequests[0].action.asDecrease.toNumber()).toBe(Number(LESS_AMOUNT));

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.cancelDelegationRequest(alith.address)
            .signAsync(ethan)
        );

        const delegationRequestsAfterCancel = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegationRequestsAfterCancel.isEmpty).toBe(true);
      },
    });
  },
});
