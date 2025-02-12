import { type DevModeContext, expect } from "@moonwall/cli";

export async function fundAccount(account: `0x${string}`, amount: bigint, context: DevModeContext) {
  await context.createBlock(context.polkadotJs().tx.balances.transferAllowDeath(account, amount), {
    allowFailures: false,
  });

  const balance = (await context.polkadotJs().query.system.account(account)).data.free.toBigInt();
  expect(balance).to.eq(amount);
}

export async function getReservedBalance(account: `0x${string}`, context: DevModeContext) {
  return (await context.polkadotJs().query.system.account(account)).data.reserved.toBigInt();
}
