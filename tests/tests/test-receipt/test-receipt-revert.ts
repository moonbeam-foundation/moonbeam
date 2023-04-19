import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Receipt - Revert", (context) => {
  it("should generate a receipt", async function () {
    const { rawTx } = await createContract(context, "FailingConstructor");
    const { result } = await context.createBlock(rawTx);
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.be.false;
    expect(receipt).to.include({
      blockNumber: 1,
      contractAddress: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
      cumulativeGasUsed: 54602,
      from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
      gasUsed: 54602,
      to: null,
      transactionHash: result.hash,
      transactionIndex: 0,
    });
  });
});
