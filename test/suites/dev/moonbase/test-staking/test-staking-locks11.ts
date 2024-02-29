import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, KeyringPair, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";
import { chunk } from "../../../../helpers";

describeSuite({
  id: "D013374",
  title: "Staking - Locks - bottom delegator removed",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();
    let additionalDelegators: KeyringPair[];

    beforeAll(async function () {
      const maxDelegations =
        context.polkadotJs().consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber() +
        context.polkadotJs().consts.parachainStaking.maxBottomDelegationsPerCandidate.toNumber();

      // Create the delegators to fill the lists
      additionalDelegators = new Array(maxDelegations).fill(0).map(() => generateKeyringPair());

      await context.createBlock(
        [randomAccount, ...additionalDelegators].map(
          (account, i) =>
            context
              .polkadotJs()
              .tx.balances.transferAllowDeath(account.address, MIN_GLMR_DELEGATOR + 10n * GLMR)
              .signAsync(alith, { nonce: i }),
          { allowFailures: false }
        )
      );
    }, 20000);

    it({
      id: "T01",
      title: "should get removed when bumped out of bottom list",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              1,
              1,
              1
            )
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        // Additional check
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(locks.length).to.be.equal(
          1,
          `Unexpected number of locks: ${locks.map((l) => l.id.toString()).join(` - `)}`
        );

        const txns = await [...additionalDelegators].map((account, i) =>
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR + GLMR,
              100,
              additionalDelegators.length + 1,
              additionalDelegators.length + 1,
              1
            )
            .signAsync(account)
        );

        // this can no longer fit in one block
        for (const txnChunk of chunk(txns, 15)) {
          await context.createBlock(txnChunk, { allowFailures: false });
        }

        const alithCandidateInfo = (
          (await context.polkadotJs().query.parachainStaking.candidateInfo(alith.address)) as any
        ).unwrap();
        expect(alithCandidateInfo.delegationCount.toNumber()).to.equal(additionalDelegators.length);

        const newLocks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(newLocks.length).to.be.equal(
          0,
          `Unexpected number of locks: ${newLocks
            .map((l) => `${l.id.toString()}: ${l.amount.toHuman().toString()}`)
            .join(` - `)}`
        );
      },
    });
  },
});
