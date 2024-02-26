import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D013359",
  title: "Staking - Rewards Auto-Compound - delegator revoke",
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
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            baltathar.address,
            MIN_GLMR_DELEGATOR,
            100,
            0,
            0,
            1
          )
          .signAsync(ethan),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.utility.batch([
            context.polkadotJs().tx.parachainStaking.scheduleRevokeDelegation(alith.address),
            context.polkadotJs().tx.parachainStaking.scheduleRevokeDelegation(baltathar.address),
          ])
          .signAsync(ethan),
        { allowFailures: false }
      );
      const roundDelay = context
        .polkadotJs()
        .consts.parachainStaking.revokeDelegationDelay.toNumber();
      await jumpRounds(context, roundDelay);
    });

    it({
      id: "T01",
      title: "should remove all auto-compound configs across multiple candidates",
      test: async () => {
        const autoCompoundDelegationsAlithBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

        await context.createBlock(
          context
            .polkadotJs()
            .tx.utility.batch([
              context
                .polkadotJs()
                .tx.parachainStaking.executeDelegationRequest(ethan.address, alith.address),
              context
                .polkadotJs()
                .tx.parachainStaking.executeDelegationRequest(ethan.address, baltathar.address),
            ])
            .signAsync(ethan),
          { allowFailures: false }
        );

        const autoCompoundDelegationsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithAfter.toJSON()).to.be.empty;
        expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.be.empty;
      },
    });
  },
});
