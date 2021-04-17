import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Frontier RPC (Pending Pool)", `simple-specs.json`, (context) => {
  it("should return a pending transaction", async function () {
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: (await getCompiled("TestContract")).byteCode,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    const txHash = (
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).result;

    const pendingTransaction = (
      await customRequest(context.web3, "eth_getTransactionByHash", [txHash])
    ).result;
    // pending transactions do not know yet to which block they belong to
    expect(pendingTransaction).to.include({
      blockNumber: null,
      hash: txHash,
      publicKey:
        "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529f" +
        "a2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
      r: "0xeab0158195d611eb22d4f5a5788409a153b86a4c09661d469a6453b1272704ff",
      s: "0x17f220c16a8c11b07d3f1284abf483e330217807632fa93fecf92b48e817875a",
      v: "0xa26",
    });

    await createAndFinalizeBlock(context.polkadotApi);

    const processedTransaction = (
      await customRequest(context.web3, "eth_getTransactionByHash", [txHash])
    ).result;
    expect(processedTransaction).to.include({
      blockNumber: "0x1",
      hash: txHash,
      publicKey:
        "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529f" +
        "a2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
      r: "0xeab0158195d611eb22d4f5a5788409a153b86a4c09661d469a6453b1272704ff",
      s: "0x17f220c16a8c11b07d3f1284abf483e330217807632fa93fecf92b48e817875a",
      v: "0xa26",
    });
  });
});
