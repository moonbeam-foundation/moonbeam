import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  type KeyringPair,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";
import { chunk, jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D013450",
  title: "Staking - Rewards Auto-Compound - PoV Size",
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

      // Setup round and staking parameters
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
            .signAsync(alith),
        ],
        { allowFailures: false }
      );

      // Setup all new delegators accounts
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

      // fill all delegations, we split this into multiple blocks as it will not fit into one.
      // we use a maxDelegationCount here, since the transactions can come out of order.
      for (const delChunk of chunk(otherDelegators, 8)) {
        await context.createBlock(
          delChunk.map((d, i) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegateWithAutoCompound(
                alith.address,
                MIN_GLMR_DELEGATOR + 10n * GLMR,
                100,
                maxDelegationCount,
                maxDelegationCount,
                1
              )
              .signAsync(d)
          ),
          { allowFailures: false }
        );
      }
    });

    it({
      id: "T01",
      title: "should be under the limit of 3_750_000",
      test: async () => {
        // Moves to the next payout block
        await jumpRounds(context, 2);
        const { block } = await context.createBlock();

        const weights = await context.pjsApi.query.system.blockWeight();
        expect(
          weights.mandatory.proofSize.toNumber(),
          "proofSize is too low, probably missing payout in the block"
        ).to.be.at.least(100_000);

        // block could support ~5_000_000 bytes proofSize in total
        // but we consider it safer to error when reaching 55% of the capacity
        // of the block (~2_750_000) which is already high for a payout
        expect(
          weights.mandatory.proofSize.toNumber(),
          "proofSize is too high, this might lead to empty block"
        ).to.be.at.most(2_616_200);

        // block could support ~500ms refTime but we consider it safer to error when reaching
        // over 212ms for the payout. After Async Backing a block could support 2000ms.
        expect(
          weights.mandatory.refTime.toNumber(),
          "refTime over 212ms, very high for a payout"
        ).to.be.at.most(212_000_000_000);

        expect(
          weights.mandatory.proofSize.toNumber(),
          "estimated weight proofSize is under real proofSize, should never happen!"
        ).to.be.at.least(block.proofSize!);
      },
    });
  },
});
