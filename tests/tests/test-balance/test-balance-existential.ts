import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, generateKeyingPair } from "../../util/accounts";
import { GLMR } from "../../util/constants";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  const randomAccount = generateKeyingPair();
  it("setup accounts", async function () {
    await context.createBlock(
      createTransfer(context, randomAccount.address, 10n * GLMR, {
        from: alith.address,
        gas: 21000,
      })
    );
  });

  it("should be disabled (no reaped account on 0 balance)", async function () {
    await context.createBlock(
      createTransfer(context, alith.address, 10n * GLMR, {
        from: randomAccount.address,
        gas: 21000,
      })
    );
    expect(parseInt(await context.web3.eth.getBalance(randomAccount.address))).to.eq(0);
    expect(await context.web3.eth.getTransactionCount(randomAccount.address)).to.eq(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  const randomAccount = generateKeyingPair();
  it("setup accounts", async function () {
    await context.createBlock(
      createTransfer(context, randomAccount.address, 10n * GLMR, {
        from: alith.address,
        gas: 21000,
      })
    );
  });

  it("should be disabled (no reaped account on tiny balance - 1)", async function () {
    await context.createBlock(
      createTransfer(context, baltathar.address, 10n * GLMR - 1n - 21000n * 1_000_000_000n, {
        from: randomAccount.address,
        gas: 21000,
      })
    );
    expect(parseInt(await context.web3.eth.getBalance(randomAccount.address))).to.eq(1);
    expect(await context.web3.eth.getTransactionCount(randomAccount.address)).to.eq(1);
  });
});

describeDevMoonbeam("Existential Deposit", (context) => {
  it("checks that existantial deposit is set to zero", async function () {
    // Grab existential deposit
    let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;
    expect(existentialDeposit.toBigInt()).to.eq(0n);
  });
});
