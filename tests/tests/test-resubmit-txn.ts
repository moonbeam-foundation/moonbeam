import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../util/constants";
import { createTransfer } from "../util/transactions";
import { customWeb3Request } from "../util/providers";

const testAccount = "0x1111111111111111111111111111111111111111";

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should allow resubmitting with higher gas", async function () {
    const optionsLowGas = { nonce: 0, gasPrice: 0 };
    const optionsHighGas = { nonce: 0, gasPrice: 1 };

    const transactions = [
      await createTransfer(context, testAccount, 1, optionsLowGas),
      await createTransfer(context, testAccount, 2, optionsHighGas),
    ];
    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(testAccount, 1)).to.equal((2).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should ignore resubmitting with lower gas", async function () {
    const optionsLowGas = { nonce: 0, gasPrice: 0 };
    const optionsHighGas = { nonce: 0, gasPrice: 1 };

    const transactions = [
      await createTransfer(context, testAccount, 3, optionsHighGas),
      await createTransfer(context, testAccount, 1, optionsLowGas),
    ];
    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(testAccount, 1)).to.equal((3).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should allow cancelling transaction", async function () {
    // gas price should trump limit
    const optionsLowGas = { nonce: 0, gasPrice: 0, gas: 0xfffff };
    const optionsHighGas = { nonce: 0, gasPrice: 1, gas: 0x10000 };

    const transactions = [
      await createTransfer(context, testAccount, 1, optionsLowGas),
      await createTransfer(context, testAccount, 2, optionsHighGas),
    ];
    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(testAccount, 1)).to.equal((2).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should pick highest gas price from many transactions", async function () {
    const optionsHighGas = { nonce: 0, gasPrice: 100 }; // gas price should trump limit

    let transactions = [];
    for (let i = 1; i < 20; i++) {
      const options = { nonce: 0, gasPrice: i };
      transactions.push(await createTransfer(context, testAccount, i * 10, options));
    }

    // our expected txn...
    transactions.push(await createTransfer(context, testAccount, 42, optionsHighGas));

    for (let i = 1; i < 20; i++) {
      const options = { nonce: 0, gasPrice: i + 30 };
      transactions.push(await createTransfer(context, testAccount, i * 100, options));
    }

    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(testAccount, 1)).to.equal((42).toString());
  });
});
