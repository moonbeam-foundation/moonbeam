import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { TransactionReceipt } from "web3-core";

import { alith } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Ethereum RPC - Filtering non-matching logs", (context) => {
  let non_matching_cases: ReturnType<typeof getNonMatchingCases> = null;
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
    const { rawTx } = await createContract(context, "EventEmitter", {
      from: alith.address,
    });
    const { result } = await context.createBlock(rawTx);
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
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
  it("EthApi::getLogs - should return `unknown block`.", async function () {
    let request = await customWeb3Request(context.web3, "eth_getLogs", [
      {
        blockHash: "0x1234000000000000000000000000000000000000000000000000000000000000",
      },
    ]);
    expect(request.error.message).to.be.equal("unknown block");
    expect(request.error.code).to.be.equal(-32000);
  });
});
