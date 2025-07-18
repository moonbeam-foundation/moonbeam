import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  type KeyringPair,
  MILLIGLMR,
  MIN_GLMR_DELEGATOR,
  alith,
  generateKeyringPair,
} from "@moonwall/util";
import { chunk, getDelegatorStakingFreeze } from "../../../../helpers";

describeSuite({
  id: "D023477",
  title: "Staking - Freezes - bottom and top delegations",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let bottomDelegators: KeyringPair[];
    let topDelegators: KeyringPair[];

    beforeAll(async function () {
      // Create the delegators to fill the lists
      bottomDelegators = new Array(
        context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber()
      )
        .fill(0)
        .map(() => generateKeyringPair());
      topDelegators = new Array(
        context.polkadotJs().consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber()
      )
        .fill(0)
        .map(() => generateKeyringPair());

      await context.createBlock(
        [...bottomDelegators, ...topDelegators].map((account, i) =>
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(account.address, MIN_GLMR_DELEGATOR + 20n * GLMR)
            .signAsync(alith, { nonce: i })
        ),
        { allowFailures: false }
      );
    }, 20000);

    it({
      id: "T01",
      title: "should be set for bottom and top list delegators",
      test: async function () {
        let tipOrdering = topDelegators.length + 1;
        let numDelegations = 0;
        for (const topDelegatorsChunk of chunk(topDelegators, 20)) {
          await context.createBlock(
            [...topDelegatorsChunk].map((account, i) => {
              // add a tip such that the delegation ordering will be preserved,
              // e.g. the first txns sent will have the highest tip
              const tip = BigInt(tipOrdering--) * MILLIGLMR;
              return context
                .polkadotJs()
                .tx.parachainStaking.delegateWithAutoCompound(
                  alith.address,
                  MIN_GLMR_DELEGATOR + 1n * GLMR,
                  100,
                  numDelegations,
                  numDelegations++,
                  1
                )
                .signAsync(account, { tip });
            }),
            { allowFailures: false }
          );
        }

        // allow more block(s) for txns to be processed...
        // note: this only seems necessary when a tip is added, otherwise all 300 txns make it into
        // a single block. A tip is necessary if the txns are not otherwise executed in order of
        // submission, which is highly dependent on txpool prioritization logic.

        // TODO: it would be good to diagnose this further: why does adding a tip appear to reduce
        // the number of txns included?
        const numBlocksToWait = 1;
        let numBlocksWaited = 0;
        while (numBlocksWaited < numBlocksToWait) {
          await context.createBlock();
          // Check freezes instead of locks for top delegators
          const freezesPromises = topDelegators.map((delegator) =>
            getDelegatorStakingFreeze(delegator.address as `0x${string}`, context)
          );
          const topFreezes = await Promise.all(freezesPromises);
          const numDelegatorFreezes = topFreezes.filter((freeze) => freeze > 0n).length;

          if (numDelegatorFreezes < topDelegators.length) {
            numBlocksWaited += 1;
            expect(numBlocksWaited).to.be.lt(
              numBlocksToWait,
              "Top delegation extrinsics not included in time"
            );
          } else {
            expect(numDelegatorFreezes).to.eq(
              topDelegators.length,
              "More delegations than expected"
            );
            break;
          }
        }

        tipOrdering = bottomDelegators.length + 1;
        numDelegations = topDelegators.length;
        for (const bottomDelegatorsChunk of chunk(bottomDelegators, 20)) {
          await context.createBlock(
            [...bottomDelegatorsChunk].map((account) => {
              // add a tip such that the delegation ordering will be preserved,
              // e.g. the first txns sent will have the highest tip
              const tip = BigInt(tipOrdering--) * MILLIGLMR;
              return context
                .polkadotJs()
                .tx.parachainStaking.delegateWithAutoCompound(
                  alith.address,
                  MIN_GLMR_DELEGATOR,
                  100,
                  numDelegations,
                  numDelegations++,
                  1
                )
                .signAsync(account, { tip });
            }),
            { allowFailures: false }
          );
        }

        // note that we don't need to wait for further blocks here because bottom delegations is
        // much smaller than top delegations, so all txns reliably fit within one block.
        const bottomFreezesPromises = bottomDelegators.map((delegator) =>
          getDelegatorStakingFreeze(delegator.address as `0x${string}`, context)
        );
        const bottomFreezes = await Promise.all(bottomFreezesPromises);
        const numBottomDelegatorFreezes = bottomFreezes.filter((freeze) => freeze > 0n).length;
        expect(numBottomDelegatorFreezes).to.equal(
          context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber()
        );
      },
    });
  },
});
