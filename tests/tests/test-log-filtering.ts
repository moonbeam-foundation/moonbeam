import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import { createContract } from "../util/transactions";
import { GENESIS_ACCOUNT } from "../util/constants";
import { TransactionReceipt } from "web3-core";

describeDevMoonbeam("Log - Filter out non-matching", (context) => {
  let non_matching_cases = null;
  function getNonMatchingCases(receipt: TransactionReceipt) {
    return [
      // Non-existant address.
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: "0x0000000000000000000000000000000000000000",
      },
      // Non-existant topic.
      {
        fromBlock: "0x0",
        toBlock: "latest",
        topics: ["0x0000000000000000000000000000000000000000000000000000000000000000"],
      },
      // Existant address + non-existant topic.
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: receipt.contractAddress,
        topics: ["0x0000000000000000000000000000000000000000000000000000000000000000"],
      },
      // Non-existant address + existant topic.
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: "0x0000000000000000000000000000000000000000",
        topics: receipt.logs[0].topics,
      },
    ];
  }
  before("Setup: Create block with transfer", async () => {
    const { rawTx } = await createContract(context, "SingleEventContract", {
      from: GENESIS_ACCOUNT,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);
    non_matching_cases = getNonMatchingCases(receipt);
  });
  it("EthFilterApi::getFilterLogs - should filter out non-matching cases.", async function () {
    let create_filter;
    for (var item of non_matching_cases) {
      create_filter = await customWeb3Request(context.web3, "eth_newFilter", [item]);
      let poll = await customWeb3Request(context.web3, "eth_getFilterLogs", [create_filter.result]);
      expect(poll.result.length).to.be.eq(0);
    }
  });
  it("EthApi::getLogs - should filter out non-matching cases.", async function () {
    for (var item of non_matching_cases) {
      let request = await customWeb3Request(context.web3, "eth_getLogs", [item]);
      expect(request.result.length).to.be.eq(0);
    }
  });
});
