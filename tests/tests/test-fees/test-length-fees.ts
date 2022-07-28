import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { baltathar } from "../../util/accounts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonbase)",
  (context) => {
    it("should have low balance transfer fees", async () => {
      const fee = await testBalanceTransfer(context);
      expect(fee).to.equal(12772901520875n);
    });
  },
  "Legacy",
  "moonbase"
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonbase)",
  (context) => {
    it("should have expensive runtime-upgrade fees", async () => {
      const fee = await testRuntimeUpgrade(context);
      expect(fee).to.equal(9226793130623667008n);
    });
  },
  "Legacy",
  "moonbase"
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonriver)",
  (context) => {
    it("should have low balance transfer fees", async () => {
      const fee = await testBalanceTransfer(context);
      expect(fee).to.equal(12772901520875n);
    });
  },
  "Legacy",
  "moonriver"
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonriver)",
  (context) => {
    it("should have expensive runtime-upgrade fees", async () => {
      const fee = await testRuntimeUpgrade(context);
      expect(fee).to.equal(9226793130623667008n);
    });
  },
  "Legacy",
  "moonriver"
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonbeam)",
  (context) => {
    it("should have low balance transfer fees", async () => {
      const fee = await testBalanceTransfer(context);
      expect(fee).to.equal(1277290152087500n);
    });
  },
  "Legacy",
  "moonbeam"
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction (Moonbeam)",
  (context) => {
    it("should have expensive runtime-upgrade fees", async () => {
      const fee = await testRuntimeUpgrade(context);
      expect(fee).to.equal(922679313062366700800n);
    });
  },
  "Legacy",
  "moonbeam"
);

// define our tests here so we can DRY.
// each test submits some txn then measures and returns the fees charged.

const testBalanceTransfer = async (context: DevTestContext) => {
  let initialBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  // send a balance transfer to self and see what our fees end up being
  await context.createBlock(
    context.polkadotApi.tx.balances.transfer(baltathar.address, 1).signAsync(baltathar)
  );

  let afterBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  const fee = initialBalance - afterBalance;
  return fee;
};

const testRuntimeUpgrade = async (context: DevTestContext) => {
  const initialBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  // generate a mock runtime upgrade hex string
  let size = 4194304; // 2MB bytes represented in hex
  let hex = "0x" + "F".repeat(size);

  // send an enactAuthorizedUpgrade. we expect this to fail, but we just want to see that it was
  // included in a block (not rejected) and was charged based on its length
  await context.polkadotApi.tx.parachainSystem.enactAuthorizedUpgrade(hex).signAndSend(baltathar);
  await context.createBlock();

  let afterBalance = (
    await context.polkadotApi.query.system.account(baltathar.address)
  ).data.free.toBigInt();

  const fee = initialBalance - afterBalance;
  return fee;
};
