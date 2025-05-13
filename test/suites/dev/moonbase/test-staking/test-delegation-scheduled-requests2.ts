import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

const numberToHex = (n: bigint): string => `0x${n.toString(16).padStart(32, "0")}`;

describeSuite({
  id: "D013426",
  title: "Staking - Delegation Scheduled Requests - cancel scheduled revoke",
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
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const currentRound = (
          await context.polkadotJs().query.parachainStaking.round()
        ).current.toNumber();
        const delegationRequestsAfterSchedule = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        const roundDelay = context
          .polkadotJs()
          .consts.parachainStaking.revokeDelegationDelay.toNumber();
        expect(delegationRequestsAfterSchedule.toJSON()).to.deep.equal([
          {
            delegator: ethan.address,
            whenExecutable: currentRound + roundDelay,
            action: {
              revoke: numberToHex(MIN_GLMR_DELEGATOR),
            },
          },
        ]);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.cancelDelegationRequest(alith.address)
            .signAsync(ethan)
        );

        const delegationRequestsAfterCancel = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegationRequestsAfterCancel).to.be.empty;
      },
    });
  },
});
