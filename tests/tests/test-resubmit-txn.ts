import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { TEST_ACCOUNT } from "../util/constants";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should allow resubmitting with higher gas", async function () {
    const optionsLowGas = { nonce: 0, gasPrice: 0 };
    const optionsHighGas = { nonce: 0, gasPrice: 1 };

    const transactions = [
      await createTransfer(context, TEST_ACCOUNT, 1, optionsLowGas),
      await createTransfer(context, TEST_ACCOUNT, 2, optionsHighGas),
    ];
    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal((2).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should ignore resubmitting with lower gas", async function () {
    const optionsLowGas = { nonce: 0, gasPrice: 0 };
    const optionsHighGas = { nonce: 0, gasPrice: 1 };

    const transactions = [
      await createTransfer(context, TEST_ACCOUNT, 3, optionsHighGas),
      await createTransfer(context, TEST_ACCOUNT, 1, optionsLowGas),
    ];
    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal((3).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should allow cancelling transaction", async function () {
    // gas price should trump limit
    const optionsLowGas = { nonce: 0, gasPrice: 0, gas: 0xfffff };
    const optionsHighGas = { nonce: 0, gasPrice: 1, gas: 0x10000 };

    const transactions = [
      await createTransfer(context, TEST_ACCOUNT, 1, optionsLowGas),
      await createTransfer(context, TEST_ACCOUNT, 2, optionsHighGas),
    ];
    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal((2).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should pick highest gas price from many transactions", async function () {
    const optionsHighGas = { nonce: 0, gasPrice: 100 }; // gas price should trump limit

    let transactions = [];
    for (let i = 1; i < 20; i++) {
      const options = { nonce: 0, gasPrice: i };
      transactions.push(await createTransfer(context, TEST_ACCOUNT, i * 10, options));
    }

    // our expected txn...
    transactions.push(await createTransfer(context, TEST_ACCOUNT, 42, optionsHighGas));

    for (let i = 1; i < 20; i++) {
      const options = { nonce: 0, gasPrice: i + 30 };
      transactions.push(await createTransfer(context, TEST_ACCOUNT, i * 100, options));
    }

    await context.createBlock({ transactions });

    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal((42).toString());
  });
});
