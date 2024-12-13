import type { DevModeContext } from "@moonwall/cli";
import {
  GLMR,
  type KeyringPair,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";

export async function createAccounts(
  context: DevModeContext,
  maxAccounts: number,
  amount: bigint = MIN_GLMR_STAKING + 1n * GLMR
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts)).fill(0).map(() => generateKeyringPair());

  let alithNonce = await context
    .viem()
    .getTransactionCount({ address: alith.address as `0x${string}` });
  await context.createBlock(
    randomAccounts.map((randomCandidate) =>
      context
        .polkadotJs()
        .tx.sudo.sudo(
          context.polkadotJs().tx.balances.forceSetBalance(randomCandidate.address, amount)
        )
        .signAsync(alith, { nonce: alithNonce++ })
    ),
    { allowFailures: false }
  );

  return randomAccounts;
}
