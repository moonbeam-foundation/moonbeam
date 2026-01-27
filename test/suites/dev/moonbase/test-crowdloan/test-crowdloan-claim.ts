import "@moonbeam-network/api-augment";
import { DOROTHY_ADDRESS, describeSuite, dorothy, expect } from "moonwall";
import { calculate_vested_amount, getAccountPayable } from "../../../../helpers/crowdloan.js";
import { jumpBlocks } from "../../../../helpers/block.js";

describeSuite({
  id: "D020701",
  title: "Crowdloan Rewards - Claim",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const EXPECTED_TOTAL_REWARD = 3_000_000n * 10n ** 18n; // 3M DEV tokens
    const INIT_PAYMENT_PERCENTAGE = 30n;

    it({
      id: "T01",
      title: "should allow claiming vested rewards after blocks pass",
      test: async () => {
        // Get initial state
        const initialPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        const initialClaimed = initialPayable!.claimedReward.toBigInt();

        // Wait for some blocks to pass
        const numberOfBlocks = 10n;
        await jumpBlocks(context, Number(numberOfBlocks));

        // Calculate expected vested amount
        const expectedInitialPayment = (EXPECTED_TOTAL_REWARD * INIT_PAYMENT_PERCENTAGE) / 100n;
        const expectedVested = await calculate_vested_amount(
          EXPECTED_TOTAL_REWARD,
          expectedInitialPayment,
          numberOfBlocks
        );

        // Claim rewards
        const balanceBefore = (
          await context.polkadotJs().query.system.account(DOROTHY_ADDRESS)
        ).data.free.toBigInt();

        await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.claim().signAsync(dorothy)
        );

        const balanceAfter = (
          await context.polkadotJs().query.system.account(DOROTHY_ADDRESS)
        ).data.free.toBigInt();

        // Verify the claim
        const finalPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        const claimedAmount = finalPayable!.claimedReward.toBigInt() - initialClaimed;

        expect(claimedAmount).toBeGreaterThan(0n);
        expect(finalPayable!.claimedReward.toBigInt()).toBeGreaterThanOrEqual(expectedVested);
        expect(balanceAfter).toBeGreaterThan(balanceBefore);
      },
    });

    it({
      id: "T02",
      title: "should allow claiming small amounts as each block vests rewards",
      test: async () => {
        // Each createBlock advances relay block by 2, which vests more rewards
        // This test verifies that even small time increments allow claiming

        const payableBefore = await getAccountPayable(context, DOROTHY_ADDRESS);
        const claimedBefore = payableBefore!.claimedReward.toBigInt();

        // Advance just one block (which increases relay block by 2)
        await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.claim().signAsync(dorothy)
        );

        const payableAfter = await getAccountPayable(context, DOROTHY_ADDRESS);
        const claimedAfter = payableAfter!.claimedReward.toBigInt();

        // Should be able to claim at least something since relay blocks advanced
        expect(claimedAfter).toBeGreaterThan(claimedBefore);
      },
    });

    it({
      id: "T03",
      title: "should allow claiming multiple times as rewards vest",
      test: async () => {
        const claims: bigint[] = [];

        // Make multiple claims over time
        for (let i = 0; i < 3; i++) {
          await jumpBlocks(context, 5);

          const payableBefore = await getAccountPayable(context, DOROTHY_ADDRESS);
          const claimedBefore = payableBefore!.claimedReward.toBigInt();

          await context.createBlock(
            context.polkadotJs().tx.crowdloanRewards.claim().signAsync(dorothy)
          );

          const payableAfter = await getAccountPayable(context, DOROTHY_ADDRESS);
          const claimedAfter = payableAfter!.claimedReward.toBigInt();

          const claimAmount = claimedAfter - claimedBefore;
          if (claimAmount > 0n) {
            claims.push(claimAmount);
          }
        }

        // Should have at least one successful claim
        expect(claims.length).toBeGreaterThan(0);
      },
    });

    it({
      id: "T04",
      title: "should never exceed total reward amount",
      test: async () => {
        // Fast forward significantly
        await jumpBlocks(context, 100);

        await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.claim().signAsync(dorothy)
        );

        const payable = await getAccountPayable(context, DOROTHY_ADDRESS);

        expect(payable!.claimedReward.toBigInt()).toBeLessThanOrEqual(
          payable!.totalReward.toBigInt()
        );
      },
    });

    it({
      id: "T05",
      title: "should emit RewardsPaid event on successful claim",
      test: async () => {
        await jumpBlocks(context, 10);

        const result = await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.claim().signAsync(dorothy)
        );

        const events = result.result?.events || [];
        const rewardsPaidEvent = events.find((e) =>
          context.polkadotJs().events.crowdloanRewards.RewardsPaid.is(e.event)
        );

        expect(rewardsPaidEvent).toBeDefined();
      },
    });
  },
});
