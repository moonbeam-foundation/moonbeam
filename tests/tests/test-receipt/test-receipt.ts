import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, baltathar } from "../../util/accounts";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Receipt - Contract", (context) => {
  let txHash;
  let eventContract;
  before("Setup: Create block with contract", async () => {
    const { contract, rawTx } = await createContract(context, "SingleEventContract", {
      from: alith.address,
    });
    const { result } = await context.createBlockWithEth(rawTx);
    txHash = result.result;
    eventContract = contract;
  });

  it("Should generate receipt", async function () {
    const block = await context.web3.eth.getBlock(1);
    const receipt = await context.web3.eth.getTransactionReceipt(txHash);

    expect(receipt.blockHash).to.be.eq(block.hash);
    expect(receipt.blockNumber).to.be.eq(block.number);
    expect(receipt.from).to.be.eq(alith.address.toLowerCase()); // web3 rpc returns lowercase
    expect(receipt.logs.length).to.be.eq(1);
    expect(receipt.logs[0].address).to.be.eq(eventContract.options.address);
    expect(receipt.logs[0].blockHash).to.be.eq(block.hash);
  });
});

describeDevMoonbeam(
  "Receipt - EIP1559",
  (context) => {
    it("should calculate effective gas price", async function () {
      const preBalance = BigInt(await context.web3.eth.getBalance(alith.address));
      // With this configuration only half of the priority fee will be used, as the max_fee_per_gas
      // is 2GWEI and the base fee is 1GWEI.
      const maxFeePerGas = 1_000_000_000 * 2;

      await context.createBlockWithEth(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          gas: "0x5208",
          maxFeePerGas: maxFeePerGas,
          maxPriorityFeePerGas: maxFeePerGas,
          to: baltathar.address,
          data: "0x",
        })
      );

      const block = await context.web3.eth.getBlock("latest");
      const receipt = await context.web3.eth.getTransactionReceipt(block.transactions[0]);
      // The receipt should contain an effective gas price of 2GWEI.
      expect(receipt.effectiveGasPrice).to.be.eq(maxFeePerGas);
    });
  },
  "EIP1559"
);
