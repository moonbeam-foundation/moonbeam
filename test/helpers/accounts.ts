import { DevModeContext } from "@moonwall/cli";
import { KeyringPair } from "@moonwall/util";
import { generateKeyringPair, alith } from "../../tests/util/accounts";
import { MIN_GLMR_STAKING, GLMR } from "../../tests/util/constants";

export async function createAccounts(
  context: DevModeContext,
  maxAccounts: number
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts)).fill(0).map(() => generateKeyringPair());

  let alithNonce = await context
    .viem("public")
    .getTransactionCount({ address: alith.address as `0x{string}` });
  await context.createBlock(
    randomAccounts.map((randomCandidate) =>
      context
        .polkadotJs()
        .tx.balances.transfer(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
        .signAsync(alith, { nonce: alithNonce++ })
    )
  );

  return randomAccounts;
}
