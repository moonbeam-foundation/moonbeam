import "@moonbeam-network/api-augment";
import { MIN_GLMR_DELEGATOR, alith, beforeAll, describeSuite, ethan, expect } from "moonwall";

describeSuite({
  id: "D023336",
  title: "Staking - Delegation Scheduled Requests - schedule bond less",
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
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR + LESS_AMOUNT,
              0,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        // Create an extra block to prevent mismatches while comparing 'whenExecutable' field.
        await context.createBlock();

        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
            .signAsync(ethan),
          { allowFailures: false }
        );

        const delegationRequestsAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address, ethan.address);
        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();

        // The delegator is implied by the storage key (alith, ethan); the
        // value now only contains the execution round and action. Assert that
        // the scheduled request matches the expected timing and amount.
        expect(delegationRequestsAfter[0].whenExecutable.toNumber()).toBe(
          currentRound + roundDelay
        );
        expect(delegationRequestsAfter[0].action.isDecrease).toBe(true);
        expect(delegationRequestsAfter[0].action.asDecrease.toNumber()).toBe(Number(LESS_AMOUNT));
      },
    });
  },
});
