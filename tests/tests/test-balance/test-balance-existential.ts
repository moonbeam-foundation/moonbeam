import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { Account } from "web3-core";
import { alith, baltathar } from "../../util/accounts";
import { GLMR, MIN_GAS_PRICE } from "../../util/constants";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  let randomWeb3Account: Account;
  it("setup accounts", async function () {
    randomWeb3Account = context.web3.eth.accounts.create("random");
    const { result, block } = await context.createBlock(
      createTransfer(context, randomWeb3Account.address, 10n * GLMR, {
        from: alith.address,
        gas: 21000,
      })
    );
    expect(result.successful, result.error?.name).to.be.true;
  });

  it("should be disabled (no reaped account on 0 balance)", async function () {
    const { block, result } = await context.createBlock(
      createTransfer(context, alith.address, 10n * GLMR - 21000n * MIN_GAS_PRICE, {
        from: randomWeb3Account.address,
        privateKey: randomWeb3Account.privateKey,
        gas: 21000,
      })
    );
    expect(result.successful, result.error?.name).to.be.true;
    expect(parseInt(await context.web3.eth.getBalance(randomWeb3Account.address))).to.eq(0);
    expect(await context.web3.eth.getTransactionCount(randomWeb3Account.address)).to.eq(1);
  });
});

describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
  let randomWeb3Account: Account;
  it("setup accounts", async function () {
    randomWeb3Account = context.web3.eth.accounts.create("random");
    await context.createBlock(
      createTransfer(context, randomWeb3Account.address, 10n * GLMR, {
        from: alith.address,
        gas: 21000,
      })
    );
  });

  it("should be disabled (no reaped account on tiny balance - 1)", async function () {
    await context.createBlock(
      createTransfer(context, baltathar.address, 10n * GLMR - 1n - 21000n * 1_000_000_000n, {
        from: randomWeb3Account.address,
        privateKey: randomWeb3Account.privateKey,
        gas: 21000,
      })
    );
    expect(parseInt(await context.web3.eth.getBalance(randomWeb3Account.address))).to.eq(1);
    expect(await context.web3.eth.getTransactionCount(randomWeb3Account.address)).to.eq(1);
  });
});

describeDevMoonbeam("Existential Deposit", (context) => {
  it("checks that existantial deposit is set to zero", async function () {
    // Grab existential deposit
    let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;
    expect(existentialDeposit.toBigInt()).to.eq(0n);
  });
});
