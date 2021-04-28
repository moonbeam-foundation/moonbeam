import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, calculateTxnDataCost } from "../util/transactions";

describeDevMoonbeam("Precompiles - sacrifice", (context) => {
  it("should be valid", async function () {
    const txCall = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x1000000",
        gasPrice: "0x01",
        to: "0x00000000000000000000000000000000000001FF",
        data: `0x0000000000000000000000000000000000000000000000000000000000005BA0`, // 23456
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

    const txnData = contract.methods.sacrifice(amount).encodeABI();

    // create and sign txn...
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: contract.options.address,
        gasPrice: "0x01",
        gas: "0x100000",
        nonce: nonce++,
        data: txnData,
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    // console.log(`encoded(${amount}) => `, contract.methods.sacrifice(amount).encodeABI());

    // send txn...
    const txnResult = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction
    ]);

    // produce a block with this txn...
    await context.createBlock({ transactions: [txnResult.result] });

    // and get receipt
    const receipt = await context.web3.eth.getTransactionReceipt(txnResult.result);

    // also calculate what the transaction data should cost
    const txnDataCost = calculateTxnDataCost(tx.rawTransaction);

    return {
      txnResult,
      receipt,
      txnDataCost,
    };

  };

  it("should be accessible from a smart contract", async function () {
    const result = await transact(132862);
    expect(result.receipt.gasUsed).to.be.greaterThan(132862);
  });

  it("should have consistent overhead", async function () {
    // this test attempts to assert that the overall txn cost of invoking our precompile wrapper
    // should be equal to (some_constant + gas_burnt_by_sacrifice).
    //
    // however, we have to take the transaction data cost into account. see calculateTxnDataCost()
    // for details.

    // obtain cost of burning 0 gas in precompile - this establishes base cost
    const zeroCostResult = await transact(0);
    const zeroCost = zeroCostResult.receipt.gasUsed - zeroCostResult.txnDataCost;

    const oneCostResult = await transact(1);
    const oneCost = oneCostResult.receipt.gasUsed - oneCostResult.txnDataCost;

    const thousandCostResult = await transact(1000);
    const thousandCost = thousandCostResult.receipt.gasUsed - thousandCostResult.txnDataCost;

    // the cost of burning one gas should be only +1 compared to burning 0 gas (ignoring data fees)
    expect(oneCost).to.equal(zeroCost + 1);

    // the cost of burning 1000 gas should be +1000 compared to burning 0 gas (ignoring data fees)
    expect(thousandCost).to.equal(zeroCost + 1000);
  });
});
