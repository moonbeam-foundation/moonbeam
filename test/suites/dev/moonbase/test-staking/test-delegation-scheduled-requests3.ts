import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers";

describeSuite({
  id: "D023428",
  title: "Staking - Delegation Scheduled Requests - execute revoke early",
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

      // jump to a round before the actual executable Round
      const delegationRequests = await context
        .polkadotJs()
        .query.parachainStaking.delegationScheduledRequests(alith.address);
      await jumpToRound(context, delegationRequests[0].whenExecutable.toNumber() - 1);
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeDelegationRequest(ethan.address, alith.address)
            .signAsync(ethan)
        );
        expect(block.result!.error!.name).to.equal("PendingDelegationRequestNotDueYet");
      },
    });
  },
});
