import { expect } from "chai";

import { generateKeyringPair } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should allow resubmitting with higher gas", async function () {
    const randomAccount = generateKeyringPair();
    const optionsLowGas = { nonce: 0, gasPrice: 0 };
    const optionsHighGas = { nonce: 0, gasPrice: 1 };

    await context.createBlock([
      createTransfer(context, randomAccount.address, 1, optionsLowGas),
      createTransfer(context, randomAccount.address, 2, optionsHighGas),
    ]);

    expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal((2).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should ignore resubmitting with lower gas", async function () {
    const randomAccount = generateKeyringPair();
    const optionsLowGas = { nonce: 0, gasPrice: 0 };
    const optionsHighGas = { nonce: 0, gasPrice: 1 };

    await context.createBlock([
      createTransfer(context, randomAccount.address, 3, optionsHighGas),
      createTransfer(context, randomAccount.address, 1, optionsLowGas),
    ]);

    expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal((3).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should allow cancelling transaction", async function () {
    const randomAccount = generateKeyringPair();
    // gas price should trump limit
    const optionsLowGas = { nonce: 0, gasPrice: 0, gas: 0xfffff };
    const optionsHighGas = { nonce: 0, gasPrice: 1, gas: 0x10000 };

    await context.createBlock([
      createTransfer(context, randomAccount.address, 1, optionsLowGas),
      createTransfer(context, randomAccount.address, 2, optionsHighGas),
    ]);

    expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal((2).toString());
  });
});

describeDevMoonbeam("Resubmit transations", (context) => {
  it.skip("should pick highest gas price from many transactions", async function () {
    const randomAccount = generateKeyringPair();
    const optionsHighGas = { nonce: 0, gasPrice: 100 }; // gas price should trump limit

    let transactions = [];
    for (let i = 1; i < 20; i++) {
      const options = { nonce: 0, gasPrice: i };
      transactions.push(await createTransfer(context, randomAccount.address, i * 10, options));
    }

    // our expected txn...
    transactions.push(await createTransfer(context, randomAccount.address, 42, optionsHighGas));

    for (let i = 1; i < 20; i++) {
      const options = { nonce: 0, gasPrice: i + 30 };
      transactions.push(await createTransfer(context, randomAccount.address, i * 100, options));
    }

    await context.createBlock(transactions);

    expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal((42).toString());
  });
});
