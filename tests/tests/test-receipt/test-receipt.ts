import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { GENESIS_ACCOUNT } from "../../util/constants";

describeDevMoonbeam("Receipt - Contract", (context) => {
  let txHash;
  let eventContract;
  before("Setup: Create block with contract", async () => {
    const { contract, rawTx } = await createContract(context.web3, "SingleEventContract", {
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
