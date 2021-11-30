import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Receipt - Revert", (context) => {
  it("should generate a receipt", async function () {
    const { rawTx } = await createContract(context, "FailContract");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);

    expect(receipt.status).to.be.false;
    expect(receipt).to.include({
      blockNumber: 1,
      contractAddress: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
      cumulativeGasUsed: 54600,
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gasUsed: 54600,
      to: null,
      transactionHash: txResults[0].result,
      transactionIndex: 0,
    });
  });
});
