import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, ALITH_GENESIS_BALANCE, baltathar } from "../../util/accounts";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  it("should be disabled (no reaped account on 0 balance)", async function () {
    await context.createBlock(
      createTransfer(context, baltathar.address, ALITH_GENESIS_BALANCE - 21000n * 1_000_000_000n, {
        from: alith.address,
        gas: 21000,
      })
    );
    expect(parseInt(await context.web3.eth.getBalance(alith.address))).to.eq(0);
    expect(await context.web3.eth.getTransactionCount(alith.address)).to.eq(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  it("should be disabled (no reaped account on tiny balance - 1)", async function () {
    await context.createBlock(
      createTransfer(
        context,
        baltathar.address,
        ALITH_GENESIS_BALANCE - 1n - 21000n * 1_000_000_000n,
        {
          from: alith.address,
          gas: 21000,
        }
      )
    );
    expect(parseInt(await context.web3.eth.getBalance(alith.address))).to.eq(1);
    expect(await context.web3.eth.getTransactionCount(alith.address)).to.eq(1);
  });
});

describeDevMoonbeam("Existential Deposit", (context) => {
  it("checks that existantial deposit is set to zero", async function () {
    // Grab existential deposit
    let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;
    expect(existentialDeposit.toBigInt()).to.eq(0n);
  });
});
