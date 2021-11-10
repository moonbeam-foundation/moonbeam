import { expect } from "chai";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Filter Block API - Polling", (context) => {
  it("should return block information", async function () {
    const createFilter = await customWeb3Request(context.web3, "eth_newBlockFilter", []);
    const block = await context.web3.eth.getBlock("latest");

    const poll = await customWeb3Request(context.web3, "eth_getFilterChanges", [
      context.web3.utils.numberToHex(createFilter.result),
    ]);
    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0]).to.be.eq(block.hash);
  });
});

describeDevMoonbeam("Filter Block API - Polling", (context) => {
  it("should not retrieve previously polled", async function () {
    const createFilter = await customWeb3Request(context.web3, "eth_newBlockFilter", []);

    await context.createBlock();
    await customWeb3Request(context.web3, "eth_getFilterChanges", [
      context.web3.utils.numberToHex(createFilter.result),
    ]);

    await context.createBlock();
    await context.createBlock();

    const poll = await customWeb3Request(context.web3, "eth_getFilterChanges", [
      context.web3.utils.numberToHex(createFilter.result),
    ]);

    const block2 = await context.web3.eth.getBlock(2);
    const block3 = await context.web3.eth.getBlock(3);

    expect(poll.result.length).to.be.eq(2);
    expect(poll.result[0]).to.be.eq(block2.hash);
    expect(poll.result[1]).to.be.eq(block3.hash);
  });
});

describeDevMoonbeam("Filter Block API - Polling", (context) => {
  it("should be empty after already polling", async function () {
    const createFilter = await customWeb3Request(context.web3, "eth_newBlockFilter", []);

    await context.createBlock();
    await customWeb3Request(context.web3, "eth_getFilterChanges", [
      context.web3.utils.numberToHex(createFilter.result),
    ]);
    const poll = await customWeb3Request(context.web3, "eth_getFilterChanges", [
      context.web3.utils.numberToHex(createFilter.result),
    ]);

    expect(poll.result.length).to.be.eq(0);
  });
});

describeDevMoonbeam("Filter Block API - Polling", (context) => {
  it("should support filtering created contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "SingleEventContract");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });

    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);

    const createFilter = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: receipt.contractAddress,
        topics: receipt.logs[0].topics,
      },
    ]);
    const poll = await customWeb3Request(context.web3, "eth_getFilterChanges", [
      createFilter.result,
    ]);

    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0].address.toLowerCase()).to.be.eq(receipt.contractAddress.toLowerCase());
    expect(poll.result[0].topics).to.be.deep.eq(receipt.logs[0].topics);
  });
});
