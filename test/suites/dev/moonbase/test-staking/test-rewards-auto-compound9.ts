import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING, alith, baltathar, ethan } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D023461",
  title: "Staking - Rewards Auto-Compound - candidate leave",
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
        context.polkadotJs().tx.parachainStaking.scheduleLeaveCandidates(2).signAsync(baltathar),
        { allowFailures: false }
      );

      const roundDelay = context
        .polkadotJs()
        .consts.parachainStaking.leaveCandidatesDelay.toNumber();
      await jumpRounds(context, roundDelay);
    });

    it({
      id: "T01",
      title: "should remove auto-compound config only for baltathar",
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
            .tx.parachainStaking.executeLeaveCandidates(baltathar.address, 1)
            .signAsync(ethan)
        );

        const autoCompoundDelegationsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithAfter.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.be.empty;
      },
    });
  },
});
