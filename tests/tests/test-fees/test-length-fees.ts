import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { TREASURY_ACCOUNT } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";
import { baltathar } from "../../util/accounts";

describeDevMoonbeam(
  "Substrate Length Fees - Transaction",
  (context) => {
    it("should have low balance transfer fees", async () => {
      let initialBalance = (
        (await context.polkadotApi.query.system.account(baltathar.address)) as any
      ).data.free.toBigInt();

      // send a balance transfer to self and see what our fees end up being
      await context.polkadotApi.tx.balances.transfer(baltathar.address, 1).signAndSend(baltathar);
      await context.createBlock();

      let afterBalance = (
        (await context.polkadotApi.query.system.account(baltathar.address)) as any
      ).data.free.toBigInt();

      const fee = initialBalance - afterBalance;
      expect(fee).to.equal(14_325_001_520_875n);
    });
  },
  "Legacy", // not using Ethereum, doesn't matter
  "moonbase",
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction",
  (context) => {
    it("should have low balance transfer fees", async () => {
      let initialBalance = (
        (await context.polkadotApi.query.system.account(baltathar.address)) as any
      ).data.free.toBigInt();

      // send a balance transfer to self and see what our fees end up being
      await context.polkadotApi.tx.balances.transfer(baltathar.address, 1).signAndSend(baltathar);
      await context.createBlock();

      let afterBalance = (
        (await context.polkadotApi.query.system.account(baltathar.address)) as any
      ).data.free.toBigInt();

      const fee = initialBalance - afterBalance;
      expect(fee).to.equal(28_535_001_520_875n);
    });
  },
  "Legacy", // not using Ethereum, doesn't matter
  "moonriver",
);

describeDevMoonbeam(
  "Substrate Length Fees - Transaction",
  (context) => {
    it("should have low balance transfer fees", async () => {
      let initialBalance = (
        (await context.polkadotApi.query.system.account(baltathar.address)) as any
      ).data.free.toBigInt();

      // send a balance transfer to self and see what our fees end up being
      await context.polkadotApi.tx.balances.transfer(baltathar.address, 1).signAndSend(baltathar);
      await context.createBlock();

      let afterBalance = (
        (await context.polkadotApi.query.system.account(baltathar.address)) as any
      ).data.free.toBigInt();

      const fee = initialBalance - afterBalance;
      expect(fee).to.equal(2_853_500_152_087_500n); // moonbeam
    });
  },
  "Legacy", // not using Ethereum, doesn't matter
  "moonbeam",
);
