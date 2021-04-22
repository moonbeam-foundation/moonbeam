import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

describeDevMoonbeam("Precompiles - sacrifice", (context) => {
  it("should be valid", async function () {
    const txCall = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: "0x01",
        to: "0x00000000000000000000000000000000000001FF",
        data: `0x0000000000005BA0`, // 23456
      },
    ]);

    console.log(txCall);

    // should return empty result
    expect(txCall.result).equals("0x");
  });
});

describeDevMoonbeam("Precompiles - sacrifice", (context) => {
  let contract: Contract;
  let nonce = 0;

  before("Setup: Deploy contract", async function () {
    const result = await createContract(context.web3, "SacrificeWrapper");
    await context.createBlock({ transactions: [result.rawTx] });
    contract = result.contract;

    nonce++;
  });

  // helper to send a txn to call sacrifice with a specified amount of gas.
  async function transact(amount: Number) {
    // create and sign txn...
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: contract.options.address,
        gas: "0x100000",
        nonce: nonce++,
        data: contract.methods.sacrifice(amount).encodeABI(),
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    console.log("encoded => ", contract.methods.sacrifice(amount).encodeABI());

    // send txn...
    const txnResult = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction
    ]);

    // produce a block with this txn...
    await context.createBlock({ transactions: [txnResult.result] });

    // and get receipt
    const receipt = await context.web3.eth.getTransactionReceipt(txnResult.result);

    return {
      txnResult,
      receipt,
    };

  };

  it("should be accessible from a smart contract", async function () {
    const result = await transact(23457);
    expect(result.receipt.gasUsed).to.be.greaterThan(23457);
  });

  it("should have consistent overhead", async function () {
    let zeroCostResult = await transact(0);
    let thousandCostResult = await transact(1000);

    // console.log("zero => ", zeroCostResult);
    // console.log("thou => ", thousandCostResult);
    expect(zeroCostResult.receipt.gasUsed).to.equal(thousandCostResult.receipt.gasUsed - 1000);
  });
});
