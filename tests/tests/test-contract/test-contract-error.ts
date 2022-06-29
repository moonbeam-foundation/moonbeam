import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { TransactionReceipt } from "web3-core";

import { verifyLatestBlockFees } from "../../util/block";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Contract loop error", (context) => {
  it("should return OutOfGas on inifinite loop call", async function () {
    const { contract, rawTx } = await createContract(context, "Looper");
    await context.createBlock(rawTx);

    await contract.methods
      .infinite()
      .call({ gas: 12_000_000 })
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err: { message: string }) =>
        expect(err.message).to.equal(`Returned error: out of gas`)
      );
  });
});

describeDevMoonbeamAllEthTxTypes("Contract loop error", (context) => {
  it("should fail with OutOfGas on infinite loop transaction", async function () {
    const { contract, rawTx } = await createContract(context, "Looper");
    const infiniteTx = createContractExecution(
      context,
      {
        contract,
        contractCall: contract.methods.infinite(),
      },
      { nonce: 1 }
    );

    const { result } = await context.createBlock([rawTx, infiniteTx]);

    const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(
      result[1].hash
    );
    expect(receipt.status).to.eq(false);
  });
});

describeDevMoonbeamAllEthTxTypes("Contract loop error - check fees", (context) => {
  it("should fail with OutOfGas on infinite loop transaction - check fees", async function () {
    const { contract, rawTx } = await createContract(context, "Looper");
    const infiniteTx = await createContractExecution(
      context,
      {
        contract,
        contractCall: contract.methods.infinite(),
      },
      { nonce: 1 }
    );

    await context.createBlock(rawTx);
    await context.createBlock(infiniteTx);
    await verifyLatestBlockFees(context);
  });
});
