import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers";

describeSuite({
  id: "D023422",
  title: "Staking - Delegation Scheduled Requests -execute bond less exact round",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const LESS_AMOUNT = 10n;

    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar)
      );
      await context.createBlock([
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
      ]);
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            baltathar.address,
            MIN_GLMR_DELEGATOR + LESS_AMOUNT,
            0,
            0,
            0,
            1
          )
          .signAsync(ethan)
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT)
          .signAsync(ethan)
      );

      // jump to exact executable Round
      const delegationRequests = await context
        .polkadotJs()
        .query.parachainStaking.delegationScheduledRequests(alith.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeDelegationRequest(ethan.address, alith.address)
            .signAsync(ethan)
        );
        const delegatorState = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(ethan.address);
        const delegationRequestsAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegatorState.unwrap().delegations[0].owner.toString()).toBe(baltathar.address);
        expect(delegatorState.unwrap().delegations[0].amount.toBigInt()).toBe(
          MIN_GLMR_DELEGATOR + LESS_AMOUNT
        );
        expect(delegatorState.unwrap().delegations[1].owner.toString()).toBe(alith.address);
        expect(delegatorState.unwrap().delegations[1].amount.toBigInt()).toBe(MIN_GLMR_DELEGATOR);
        expect(delegationRequestsAfter.isEmpty).toBe(true);
      },
    });
  },
});
