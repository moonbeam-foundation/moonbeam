/// Fetch balance of provided account before and after the inner function is executed and
/// check it matches expected difference.
export async function expectBalanceDifference(context, address, diff, inner, expect) {
  const balance_before = await context.web3.eth.getBalance(address);

  await inner();

  const balance_after = await context.web3.eth.getBalance(address);
  expect(BigInt(balance_after)).to.be.eq(BigInt(balance_before) + BigInt(diff));
}
