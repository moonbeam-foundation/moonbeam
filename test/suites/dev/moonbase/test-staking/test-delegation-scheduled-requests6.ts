import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers";

describeSuite({
  id: "D023434",
  title: "Staking - Delegation Scheduled Requests - execute revoke on last delegation",
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
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
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
          .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
          .signAsync(ethan)
      );

      const delegationRequests = await context
        .polkadotJs()
        .query.parachainStaking.delegationScheduledRequests(alith.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber());
    });

    it({
      id: "T01",
      title: "should succeed and leave as delegator",
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
        expect(delegatorState.isNone).to.be.true; // last delegation revoked, so delegator left
        expect(delegationRequestsAfter.isEmpty).toBe(true);
      },
    });
  },
});
