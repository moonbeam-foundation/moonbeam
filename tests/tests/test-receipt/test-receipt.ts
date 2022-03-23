import { expect } from "chai";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract, createTransaction } from "../../util/transactions";
import { GENESIS_ACCOUNT, ALITH, ALITH_PRIV_KEY, TEST_ACCOUNT } from "../../util/constants";

describeDevMoonbeamAllEthTxTypes("Receipt - Contract", (context) => {
  let txHash;
  let eventContract;
  before("Setup: Create block with contract", async () => {
    const { contract, rawTx } = await createContract(context, "SingleEventContract", {
      from: GENESIS_ACCOUNT,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    txHash = txResults[0].result;
    eventContract = contract;
  });

  it("Should generate receipt", async function () {
    const block = await context.web3.eth.getBlock(1);
    const receipt = await context.web3.eth.getTransactionReceipt(txHash);

    expect(receipt.blockHash).to.be.eq(block.hash);
    expect(receipt.blockNumber).to.be.eq(block.number);
    expect(receipt.from).to.be.eq(GENESIS_ACCOUNT.toLowerCase()); // web3 rpc returns lowercase
    expect(receipt.logs.length).to.be.eq(1);
    expect(receipt.logs[0].address).to.be.eq(eventContract.options.address);
    expect(receipt.logs[0].blockHash).to.be.eq(block.hash);
  });
});

describeDevMoonbeam(
  "Receipt - EIP1559",
  (context) => {
    it("should calculate effective gas price", async function () {
      const preBalance = BigInt(await context.web3.eth.getBalance(ALITH));
      // With this configuration only half of the priority fee will be used, as the max_fee_per_gas is
      // 2GWEI and the base fee is 1GWEI.
      const maxFeePerGas = 1_000_000_000 * 2;
      const tx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x5208",
        maxFeePerGas: maxFeePerGas,
        maxPriorityFeePerGas: maxFeePerGas,
        to: TEST_ACCOUNT,
        data: "0x",
      });

      await context.createBlock({
        transactions: [tx],
      });

      const block = await context.web3.eth.getBlock("latest");
      const receipt = await context.web3.eth.getTransactionReceipt(block.transactions[0]);
      // The receipt should contain an effective gas price of 2GWEI.
      expect(receipt.effectiveGasPrice).to.be.eq(maxFeePerGas);
    });
  },
  "EIP1559"
);
