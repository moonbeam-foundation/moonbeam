import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../helpers/block.js";

describeSuite({
  id: "D2951",
  title: "Staking - Delegator Leave Execute - manually rescheduled revoke",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
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
            .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(baltathar.address, MIN_GLMR_DELEGATOR, 0, 1)
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(ethan),
        { allowFailures: false }
      );

      const leaveDelay = context.polkadotJs().consts.parachainStaking.leaveDelegatorsDelay;
      await jumpRounds(context, leaveDelay.addn(1).toNumber());

      // cancel single revoke request
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.cancelDelegationRequest(baltathar.address)
          .signAsync(ethan),
        { allowFailures: false }
      );

      // reschedule single revoke request
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleRevokeDelegation(baltathar.address)
          .signAsync(ethan),
        { allowFailures: false }
      );

      await jumpRounds(context, leaveDelay.addn(1).toNumber());
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveDelegators(ethan.address, 2)
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.true;
        const leaveEvents = block.result!.events.reduce((acc, event) => {
          if (context.polkadotJs().events.parachainStaking.DelegatorLeft.is(event.event)) {
            acc.push({
              account: event.event.data[0].toString(),
            });
          }
          return acc;
        }, []);
        expect(leaveEvents).to.deep.equal([
          {
            account: ethan.address,
          },
        ]);
      },
    });
  },
});
