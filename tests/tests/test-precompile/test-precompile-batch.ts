import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith, ALITH_PRIVATE_KEY, baltathar, charleth } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

describeDevMoonbeamAllEthTxTypes("Batch - All functions should consume the same gas", (context) => {
  it("should consume the same gas", async function () {
    const batchInterface = new ethers.utils.Interface((await getCompiled("Batch")).contract.abi);

    // each tx have a different gas limit to ensure it doesn't impact gas used
    let batchAllTx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: "0x0000000000000000000000000000000000000808",
        gas: "0x110000",
        value: "0x00",
        nonce: 0,
        data: batchInterface.encodeFunctionData("batchAll", [
          [baltathar.address, charleth.address],
          ["1000000000000000000", "2000000000000000000"],
          [],
          [],
        ]),
      },
      ALITH_PRIVATE_KEY
    );

    let batchSomeTx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: "0x0000000000000000000000000000000000000808",
        gas: "0x120000",
        value: "0x00",
        nonce: 1,
        data: batchInterface.encodeFunctionData("batchSome", [
          [baltathar.address, charleth.address],
          ["1000000000000000000", "2000000000000000000"],
          [],
          [],
        ]),
      },
      ALITH_PRIVATE_KEY
    );

    let batchSomeUntilFailureTx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: "0x0000000000000000000000000000000000000808",
        gas: "0x130000",
        value: "0x00",
        nonce: 2,
        data: batchInterface.encodeFunctionData("batchSomeUntilFailure", [
          [baltathar.address, charleth.address],
          ["1000000000000000000", "2000000000000000000"],
          [],
          [],
        ]),
      },
      ALITH_PRIVATE_KEY
    );

    const batchAllResult = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      batchAllTx.rawTransaction,
    ]);
    const batchSomeResult = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      batchSomeTx.rawTransaction,
    ]);
    const batchSomeUntilFailureResult = await customWeb3Request(
      context.web3,
      "eth_sendRawTransaction",
      [batchSomeUntilFailureTx.rawTransaction]
    );

    await context.createBlock();

    const batchAllReceipt = await context.web3.eth.getTransactionReceipt(batchAllResult.result);
    const batchSomeReceipt = await context.web3.eth.getTransactionReceipt(batchSomeResult.result);
    const batchSomeUntilFailureReceipt = await context.web3.eth.getTransactionReceipt(
      batchSomeUntilFailureResult.result
    );

    expect(batchAllReceipt["gasUsed"]).to.equal(43932);
    expect(batchSomeReceipt["gasUsed"]).to.equal(43932);
    expect(batchSomeUntilFailureReceipt["gasUsed"]).to.equal(43932);
  });
});
