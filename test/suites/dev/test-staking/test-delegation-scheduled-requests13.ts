import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds, jumpToRound } from "../../../helpers/block.js";

describeSuite({
  id: "D2934",
  title: "Staking - Delegation Scheduled Requests - delegator leave",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith),
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR + 10n, 0, 0)
          .signAsync(ethan),
      ]);
      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(baltathar.address, MIN_GLMR_DELEGATOR + 10n, 0, 1)
          .signAsync(ethan),
      ]);
      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, 10n)
          .signAsync(ethan),
      ]);
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(baltathar.address, 10n)
          .signAsync(ethan)
      );
      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan)
      );

      const roundDelay = context
        .polkadotJs()
        .consts.parachainStaking.leaveDelegatorsDelay.toNumber();
      await jumpRounds(context, roundDelay);
    });

    it({
      id: "T01",
      title: "should remove complete scheduled requests across multiple candidates",
      test: async () => {
        const delegationRequestsAlithBefore = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        const delegationRequestsBaltatharBefore = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(baltathar.address);
        expect(delegationRequestsAlithBefore.isEmpty).toBe(false);
        expect(delegationRequestsBaltatharBefore.isEmpty).toBe(false);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveDelegators(ethan.address, 2)
            .signAsync(ethan)
        );

        const delegationRequestsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        const delegationRequestsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(baltathar.address);
        expect(delegationRequestsAlithAfter.isEmpty).toBe(true);
        expect(delegationRequestsBaltatharAfter.isEmpty).toBe(true);
      },
    });
  },
});
