import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  KeyringPair,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";
import { fromBytes } from "viem";
import { chunk } from "../../../../helpers";

describeSuite({
  id: "D013383",
  title: "Staking - Locks - max delegations",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();
    let randomCandidates: KeyringPair[];
    let maxDelegationsPerDelegator: bigint;

    beforeAll(async function () {
      maxDelegationsPerDelegator = context
        .polkadotJs()
        .consts.parachainStaking.maxDelegationsPerDelegator.toBigInt();
      randomCandidates = new Array(Number(maxDelegationsPerDelegator))
        .fill(0)
        .map(() => generateKeyringPair());

      let alithNonce = await context
        .viem()
        .getTransactionCount({ address: alith.address as `0x${string}` });
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(
              randomAccount.address,
              (MIN_GLMR_DELEGATOR + GLMR) * maxDelegationsPerDelegator
            )
            .signAsync(alith, { nonce: alithNonce++ }),
          ...randomCandidates.map((randomCandidate) =>
            context
              .polkadotJs()
              .tx.balances.transferAllowDeath(randomCandidate.address, MIN_GLMR_STAKING + GLMR)
              .signAsync(alith, { nonce: alithNonce++ })
          ),
        ],
        { allowFailures: false }
      );

      // We split the candidates since they won't fit in a single block
      for (const randomCandidatesChunk of chunk(randomCandidates, 20)) {
        await context.createBlock(
          randomCandidatesChunk.map((randomCandidate) =>
            context
              .polkadotJs()
              .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, maxDelegationsPerDelegator)
              .signAsync(randomCandidate)
          ),
          { allowFailures: false }
        );
      }

      const candidates = await context.polkadotJs().query.parachainStaking.candidateInfo.entries();
      expect(candidates.length).to.be.equal(
        Number(maxDelegationsPerDelegator) + 1,
        "Missing candidates"
      );

      let nonce = await context
        .viem()
        .getTransactionCount({ address: randomAccount.address as `0x${string}` });

      for (const randomCandidatesChunk of chunk(randomCandidates, 20)) {
        await context.createBlock(
          randomCandidatesChunk.map(
            (randomCandidate) =>
              context
                .polkadotJs()
                .tx.parachainStaking.delegateWithAutoCompound(
                  randomCandidate.address,
                  MIN_GLMR_DELEGATOR,
                  100,
                  1,
                  1,
                  maxDelegationsPerDelegator + 1n
                )
                .signAsync(randomAccount, { nonce: nonce++ }),
            { allowFailures: false }
          )
        );
      }
    });

    it({
      id: "T01",
      title: "should support 100 delegations",
      test: async function () {
        // Additional check we have still have 1 delegation
        const delegatorState = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(randomAccount.address);
        expect(delegatorState.unwrap().delegations.length).to.be.equal(
          Number(maxDelegationsPerDelegator),
          "Missing delegation"
        );
        // We should gave locked MIN_GLMR_DELEGATOR * maxDelegationsPerDelegator
        const locks = await context.polkadotJs().query.balances.locks(randomAccount.address);
        expect(locks.length).to.be.equal(1, "Missing lock");
        expect(locks[0].amount.toBigInt()).to.be.equal(
          MIN_GLMR_DELEGATOR * maxDelegationsPerDelegator
        );
        expect(fromBytes(locks[0].id.toU8a(), "string")).to.be.equal("stkngdel");
      },
    });
  },
});
