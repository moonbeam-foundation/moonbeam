import { expect } from "chai";

import { TransactionReceipt } from "web3-core";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeam("Contract loop error", (context) => {
  it("should return OutOfGas on inifinite loop call", async function () {
    const { contract, rawTx } = await createContract(context.web3, "InfiniteContract");
    await context.createBlock({ transactions: [rawTx] });

    await contract.methods
      .infinite()
      .call({ gas: 12_000_000 })
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err) => expect(err.message).to.equal(`Returned error: out of gas`));
  });
});

describeDevMoonbeam("Contract loop error", (context) => {
  it("should fail with OutOfGas on infinite loop transaction", async function () {
    const { contract, rawTx } = await createContract(context.web3, "InfiniteContract");
    const infiniteTx = await createContractExecution(
      context.web3,
      {
        contract,
        contractCall: contract.methods.infinite(),
      },
      { nonce: 1 }
    );

    const { txResults } = await context.createBlock({
      transactions: [rawTx, infiniteTx],
    });

    const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(
      txResults[1].result
    );
    expect(receipt.status).to.eq(false);
  });
});
