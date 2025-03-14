import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  type KeyringPair,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  ethan,
  generateKeyringPair,
} from "@moonwall/util";
import { chunk } from "../../../../helpers";

describeSuite({
  id: "D013452",
  title: "Staking - Rewards Auto-Compound - bottom delegation kick",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let newDelegator: KeyringPair;

    beforeAll(async () => {
      const maxDelegationCount =
        context.polkadotJs().consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
        context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();
      const [delegator, ...otherDelegators] = new Array(maxDelegationCount)
        .fill(0)
        .map(() => generateKeyringPair());
      newDelegator = delegator;

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
              0,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { allowFailures: false }
      );

      let alithNonce = await context
        .viem()
        .getTransactionCount({ address: alith.address as `0x${string}` });
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(newDelegator.address, MIN_GLMR_STAKING)
            .signAsync(alith, { nonce: alithNonce++ }),
          ...otherDelegators.map((d) =>
            context
              .polkadotJs()
              .tx.balances.transferAllowDeath(d.address, MIN_GLMR_STAKING)
              .signAsync(alith, { nonce: alithNonce++ })
          ),
        ],
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            baltathar.address,
            MIN_GLMR_DELEGATOR,
            0,
            0,
            0,
            1
          )
          .signAsync(ethan),
        { allowFailures: false }
      );

      // fill all delegations, we split this into multiple blocks as it will not fit into one.
      // we use a maxDelegationCount here, since the transactions can come out of order.
      for (const delChunk of chunk(otherDelegators, 8)) {
        await context.createBlock(
          delChunk.map((d) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegateWithAutoCompound(
                alith.address,
                MIN_GLMR_DELEGATOR + 10n * GLMR,
                maxDelegationCount,
                0,
                0,
                1
              )
              .signAsync(d)
          )
        );
      }

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.setAutoCompound(alith.address, 100, 0, 2)
          .signAsync(ethan),
        { allowFailures: false }
      );
      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.setAutoCompound(baltathar.address, 100, 0, 2)
          .signAsync(ethan),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should remove auto-compound config only for alith",
      test: async () => {
        const autoCompoundDelegationsAlithBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharBefore = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);
        expect(autoCompoundDelegationsAlithBefore.toJSON()).to.not.be.empty;
        expect(autoCompoundDelegationsBaltatharBefore.toJSON()).to.not.be.empty;

        const maxDelegationCount =
          context.polkadotJs().consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
          context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();

        // This kicks ethan from bottom delegations for alith
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR + 10n * GLMR,
              maxDelegationCount,
              0,
              0,
              0
            )
            .signAsync(newDelegator),
          { allowFailures: false }
        );

        const autoCompoundDelegationsAlithAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(alith.address);
        const autoCompoundDelegationsBaltatharAfter = await context
          .polkadotJs()
          .query.parachainStaking.autoCompoundingDelegations(baltathar.address);

        expect(autoCompoundDelegationsAlithAfter.toJSON()).to.be.empty;
        expect(autoCompoundDelegationsBaltatharAfter.toJSON()).to.not.be.empty;
      },
    });
  },
});
