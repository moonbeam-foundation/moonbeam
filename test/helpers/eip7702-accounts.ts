import type { DevModeContext } from "@moonwall/cli";
import { parseEther } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

export async function createFundedAccount(context: DevModeContext) {
  const account = privateKeyToAccount(generatePrivateKey());
  await context.createBlock([
    context.polkadotJs().tx.balances.transferAllowDeath(account.address, parseEther("10")),
  ]);
  return account;
}