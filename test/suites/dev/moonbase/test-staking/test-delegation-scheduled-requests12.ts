import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers";

describeSuite({
  id: "D013425",
  title: "Staking - Delegation Scheduled Requests - collator leave",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let whenExecutable: number;
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
          .tx.parachainStaking.delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR + 10n, 0, 0, 0, 0)
          .signAsync(ethan),
      ]);

      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR + 10n, 0, 0, 0, 1)
          .signAsync(ethan),
      ]);
      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, 10n)
          .signAsync(ethan),
      ]);

      await context.createBlock([
        context
          .polkadotJs()
          .tx.parachainStaking.scheduleDelegatorBondLess(baltathar.address, 10n)
          .signAsync(ethan),
        context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(baltathar),
      ]);

      const currentRound = (
        await context.polkadotJs().query.parachainStaking.round()
      ).current.toNumber();
      const roundDelay = context
        .polkadotJs()
        .consts.parachainStaking.revokeDelegationDelay.toNumber();
      whenExecutable = currentRound + roundDelay;

      const collatorState = await context
        .polkadotJs()
        .query.parachainStaking.candidateInfo(baltathar.address);
      await jumpToRound(context, collatorState.unwrap().status.asLeaving.toNumber());
    });

    it({
      id: "T01",
      title: "should remove complete storage item",
      test: async () => {
        const delegationRequestsBefore = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(baltathar.address);
        expect(delegationRequestsBefore.toJSON()).to.not.be.empty;

        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.executeLeaveCandidates(baltathar.address, 1)
            .signAsync(ethan)
        );

        const delegationRequestsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(baltathar.address);
        const delegationRequestsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests(alith.address);
        expect(delegationRequestsAlithAfter.toJSON()).to.deep.equal([
          {
            delegator: ethan.address,
            whenExecutable,
            action: {
              decrease: 10,
            },
          },
        ]);
        expect(delegationRequestsBaltatharAfter.toJSON()).to.be.empty;
        const delagationRequestsKeys = await context
          .polkadotJs()
          .query.parachainStaking.delegationScheduledRequests.keys();
        expect(delagationRequestsKeys.map((k) => k.args[0].toString())).to.deep.equal([
          alith.address,
        ]);
      },
    });
  },
});
