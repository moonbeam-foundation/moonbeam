import { expect } from "chai";

import { createContract, createTransfer } from "../../util/transactions";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { customWeb3Request } from "../../util/providers";
import { TEST_ACCOUNT } from "../../util/constants";

describeDevMoonbeamAllEthTxTypes("EthPool - Future Ethereum transaction", (context) => {
  let txHash;
  before("Setup: Create a block with transaction", async () => {
    const { rawTx } = await createContract(context, "TestContract", {
      nonce: 1,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    txHash = txResults[0].result;
  });

  it("should not be executed until condition is met", async function () {
    const transaction = await context.web3.eth.getTransaction(txHash);
    expect(transaction.blockNumber).to.be.null;
  });

  // TODO: This is a test once we implement txpool "queued" for ethereum future transaction
  it.skip("should appear in the txpool", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_content", []);
    expect(inspect.result.pending).to.be.empty;
    expect(inspect.result.queued).to.not.be.empty;
  });
});

describeDevMoonbeamAllEthTxTypes("EthPool - Future Ethereum transaction", (context) => {
  let txHash;
  before("Setup: Create a block with transaction", async () => {
    const { rawTx } = await createContract(context, "TestContract", {
      nonce: 1,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    txHash = txResults[0].result;
  });

  it("should be executed after condition is met", async function () {
    // Create block including transaction with nonce 0
    await context.createBlock({
      transactions: [
        await createTransfer(context, TEST_ACCOUNT, 512, {
          nonce: 0,
        }),
      ],
    });

    const transaction = await context.web3.eth.getTransaction(txHash);
    expect(transaction.blockNumber).to.not.be.null;
  });
});
