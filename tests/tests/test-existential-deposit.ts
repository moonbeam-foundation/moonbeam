import { expect } from "chai";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_BALANCE } from "../util/constants";

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  it("should be disabled (no reaped account on 0 balance)", async function () {
    await context.createBlock({
      transactions: [
        await createTransfer(
          context,
          "0x1111111111111111111111111111111111111111",
          GENESIS_ACCOUNT_BALANCE - 21000n * 1_000_000_000n,
          {
            from: GENESIS_ACCOUNT,
            gas: 21000,
          }
        ),
      ],
    });
    expect(parseInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT))).to.eq(0);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  it("should be disabled (no reaped account on tiny balance - 1)", async function () {
    await context.createBlock({
      transactions: [
        await createTransfer(
          context,
          "0x1111111111111111111111111111111111111111",
          GENESIS_ACCOUNT_BALANCE - 1n - 21000n * 1_000_000_000n,
          {
            from: GENESIS_ACCOUNT,
            gas: 21000,
          }
        ),
      ],
    });
    expect(parseInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT))).to.eq(1);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(1);
  });
});

describeDevMoonbeam("Existential Deposit", (context) => {
  it("checks that existantial deposit is set to zero", async function () {
    // Grab existential deposit
    let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;
    expect(existentialDeposit.toBigInt()).to.eq(0n);
  });
});
