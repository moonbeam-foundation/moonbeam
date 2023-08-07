import "@moonbeam-network/api-augment";
import { DevModeContext, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, baltathar } from "@moonwall/util";

//TODO: Change these to be less literal
describeSuite({
  id: "D1606",
  title: "Substrate Length Fees",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should have low balance transfer fees",
      test: async () => {
        const fee = await testBalanceTransfer(context);
        expect(fee).toBeLessThanOrEqual(86737801520875n);
      },
    });

    it({
      id: "T02",
      title: "should have expensive runtime-upgrade fees",
      test: async () => {
        const fee = await testRuntimeUpgrade(context);
        expect(fee).toBeLessThanOrEqual(9226801665723667008n);
      },
    });
  },
});

// define our tests here so we can DRY.
// each test submits some txn then measures and returns the fees charged.

const testBalanceTransfer = async (context: DevModeContext) => {
  const initialBalance = (
    await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS)
  ).data.free.toBigInt();

  // send a balance transfer to self and see what our fees end up being
  await context.createBlock(
    context.polkadotJs().tx.balances.transfer(BALTATHAR_ADDRESS, 1).signAsync(baltathar)
  );

  const afterBalance = (
    await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS)
  ).data.free.toBigInt();

  const fee = initialBalance - afterBalance;
  return fee;
};

const testRuntimeUpgrade = async (context: DevModeContext) => {
  const initialBalance = (
    await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS)
  ).data.free.toBigInt();

  // generate a mock runtime upgrade hex string
  const size = 4194304; // 2MB bytes represented in hex
  const hex = "0x" + "F".repeat(size);

  // send an enactAuthorizedUpgrade. we expect this to fail, but we just want to see that it was
  // included in a block (not rejected) and was charged based on its length
  await context.polkadotJs().tx.parachainSystem.enactAuthorizedUpgrade(hex).signAndSend(baltathar);
  await context.createBlock();

  const afterBalance = (
    await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS)
  ).data.free.toBigInt();

  const fee = initialBalance - afterBalance;
  return fee;
};
