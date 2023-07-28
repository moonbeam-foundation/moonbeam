import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";


describeSuite({
  id: "D2944",
  title: "Staking - Delegator Leave Schedule - valid request",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should schedule revokes on all delegations",
      test: async () => {
        const delegatorState = (
          await context.polkadotJs().query.parachainStaking.delegatorState(ethan.address)
        ).unwrap();
        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();
        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.leaveDelegatorsDelay.toNumber();

        for await (const delegation of delegatorState.delegations) {
          const scheduledRequests = (await context
            .polkadotJs()
            .query.parachainStaking.delegationScheduledRequests(
              delegation.owner
            )) as unknown as any[];
          const revokeRequest = scheduledRequests.find(
            (req) => req.delegator.eq(ethan.address) && req.action.isRevoke
          );
          expect(revokeRequest).to.not.be.undefined;
          expect(revokeRequest.whenExecutable.toNumber()).to.equal(currentRound + roundDelay);
        }
      },
    });
  },
});