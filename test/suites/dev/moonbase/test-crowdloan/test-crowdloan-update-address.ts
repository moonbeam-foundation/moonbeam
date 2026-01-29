import "@moonbeam-network/api-augment";
import { BALTATHAR_ADDRESS, DOROTHY_ADDRESS, describeSuite, dorothy, expect } from "moonwall";
import { getAccountPayable } from "../../../../helpers/crowdloan.js";
import { jumpBlocks } from "../../../../helpers/block.js";

describeSuite({
  id: "D020704",
  title: "Crowdloan Rewards - Update Address",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should allow updating reward address from current account",
      test: async () => {
        // Verify Dorothy has rewards from genesis
        const initialPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        expect(initialPayable).not.toBeNull();

        const totalReward = initialPayable!.totalReward.toBigInt();
        const claimedReward = initialPayable!.claimedReward.toBigInt();

        // Update to Baltathar's address
        await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.updateRewardAddress(BALTATHAR_ADDRESS)
            .signAsync(dorothy)
        );

        // Verify old address no longer has rewards
        const oldPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        expect(oldPayable).toBeNull();

        // Verify new address has the rewards
        const newPayable = await getAccountPayable(context, BALTATHAR_ADDRESS);
        expect(newPayable).not.toBeNull();
        expect(newPayable!.totalReward.toBigInt()).toBe(totalReward);
        expect(newPayable!.claimedReward.toBigInt()).toBe(claimedReward);
      },
    });

    it({
      id: "T02",
      title: "should fail when updating to an address that already has rewards",
      test: async () => {
        // After T01, Baltathar should have the rewards
        const baltatharPayable = await getAccountPayable(context, BALTATHAR_ADDRESS);
        expect(baltatharPayable).not.toBeNull();

        const { baltathar } = await import("moonwall");

        // Try to update to the same address (should fail)
        const result = await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.updateRewardAddress(BALTATHAR_ADDRESS)
            .signAsync(baltathar)
        );

        expect(result.result?.successful).toBe(false);
      },
    });

    it({
      id: "T03",
      title: "should fail when caller has no associated rewards",
      test: async () => {
        // After T01, Dorothy no longer has rewards
        const dorothyPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        expect(dorothyPayable).toBeNull();

        // Try to update from Dorothy who has no rewards (should fail)
        const result = await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.updateRewardAddress(BALTATHAR_ADDRESS)
            .signAsync(dorothy)
        );

        expect(result.result?.successful).toBe(false);
      },
    });

    it({
      id: "T04",
      title: "should preserve contributed relay addresses after update",
      test: async () => {
        // After previous tests, Baltathar should have the rewards
        const initialPayable = await getAccountPayable(context, BALTATHAR_ADDRESS);
        expect(initialPayable).not.toBeNull();

        const relayAddresses = initialPayable!.contributedRelayAddresses;
        const { baltathar } = await import("moonwall");

        // Update address back to Dorothy
        await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.updateRewardAddress(DOROTHY_ADDRESS)
            .signAsync(baltathar)
        );

        // Check relay addresses are preserved
        const newPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        expect(newPayable).not.toBeNull();
        expect(newPayable!.contributedRelayAddresses.length).toBe(relayAddresses.length);
        expect(newPayable!.contributedRelayAddresses[0].toString()).toBe(
          relayAddresses[0].toString()
        );
      },
    });

    it({
      id: "T05",
      title: "should emit RewardAddressUpdated event",
      test: async () => {
        // After T04, Dorothy should have the rewards again
        const dorothyPayable = await getAccountPayable(context, DOROTHY_ADDRESS);
        expect(dorothyPayable).not.toBeNull();

        // Update to Baltathar and check event
        const result = await context.createBlock(
          context
            .polkadotJs()
            .tx.crowdloanRewards.updateRewardAddress(BALTATHAR_ADDRESS)
            .signAsync(dorothy)
        );

        const events = result.result?.events || [];
        const updateEvent = events.find((e) =>
          context.polkadotJs().events.crowdloanRewards.RewardAddressUpdated.is(e.event)
        );

        expect(updateEvent).toBeDefined();

        if (updateEvent) {
          const [oldAddress, newAddress] = updateEvent.event.data;
          expect(oldAddress.toString()).toBe(DOROTHY_ADDRESS);
          expect(newAddress.toString()).toBe(BALTATHAR_ADDRESS);
        }
      },
    });

    it({
      id: "T06",
      title: "should allow claiming from new address after update",
      test: async () => {
        // After T05, Baltathar should have the rewards
        const payableBefore = await getAccountPayable(context, BALTATHAR_ADDRESS);
        expect(payableBefore).not.toBeNull();

        const claimedBefore = payableBefore!.claimedReward.toBigInt();

        // Wait for some vesting
        await jumpBlocks(context, 10);

        const { baltathar } = await import("moonwall");

        // Claim from Baltathar's address
        await context.createBlock(
          context.polkadotJs().tx.crowdloanRewards.claim().signAsync(baltathar)
        );

        const payableAfter = await getAccountPayable(context, BALTATHAR_ADDRESS);
        const claimedAfter = payableAfter!.claimedReward.toBigInt();

        expect(claimedAfter).toBeGreaterThan(claimedBefore);
      },
    });
  },
});
