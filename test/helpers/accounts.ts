import type { DevModeContext } from "moonwall";
import { GLMR, MIN_GLMR_STAKING, alith, generateKeyringPair } from "moonwall";
import type { KeyringPair } from "@polkadot/keyring/types";
import { chunk } from "./common";

const MAX_FUNDED_ACCOUNTS_PER_BLOCK = 300;

export async function createAccounts(
  context: DevModeContext,
  maxAccounts: number,
  amount: bigint = MIN_GLMR_STAKING + 1n * GLMR
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts)).fill(0).map(() => generateKeyringPair());

  let alithNonce = await context
    .viem()
    .getTransactionCount({ address: alith.address as `0x${string}` });
  for (const accountChunk of chunk(randomAccounts, MAX_FUNDED_ACCOUNTS_PER_BLOCK)) {
    await context.createBlock(
      accountChunk.map((randomCandidate) =>
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context.polkadotJs().tx.balances.forceSetBalance(randomCandidate.address, amount)
          )
          .signAsync(alith, { nonce: alithNonce++ })
      ),
      { allowFailures: false }
    );
  }

  return randomAccounts;
}
