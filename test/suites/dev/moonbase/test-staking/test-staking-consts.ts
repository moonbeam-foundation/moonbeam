import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  GLMR,
  type KeyringPair,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";
import { chunk } from "../../../../helpers";

describeSuite({
  id: "D013471",
  title: "Staking - Consts - MaxDelegationsPerDelegator",
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

      let alithNonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

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
              .tx.balances.transferAllowDeath(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
              .signAsync(alith, { nonce: alithNonce++ })
          ),
        ],
        { allowFailures: false }
      );

      for (const randomCandidatesChunk of chunk(randomCandidates, 50)) {
        await context.createBlock(
          randomCandidatesChunk.map((randomCandidate) =>
            context
              .polkadotJs()
              .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, maxDelegationsPerDelegator)
              .signAsync(randomCandidate)
          )
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
          randomCandidatesChunk.map((randomCandidate) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegateWithAutoCompound(
                randomCandidate.address,
                MIN_GLMR_DELEGATOR,
                100,
                1,
                1,
                maxDelegationsPerDelegator
              )
              .signAsync(randomAccount, { nonce: nonce++ })
          )
        );
      }
    });

    it({
      id: "T01",
      title: "should fail delegation request",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegate(
              alith.address,
              MIN_GLMR_DELEGATOR,
              1,
              maxDelegationsPerDelegator + 1n
            )
            .signAsync(randomAccount)
        );
        expect(result!.successful).to.be.false;
        expect(result!.error!.name).to.be.equal("ExceedMaxDelegationsPerDelegator");
      },
    });
  },
});
