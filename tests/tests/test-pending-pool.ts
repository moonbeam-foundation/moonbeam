import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Frontier RPC (Pending Pool)", `simple-specs.json`, (context) => {
  it("should return a pending transaction", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: (await getCompiled("TEST_CONTRACT")).byteCode,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    const tx_hash = (
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).result;

    const pending_transaction = (
      await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])
    ).result;
    // pending transactions do not know yet to which block they belong to
    expect(pending_transaction).to.include({
      blockNumber: null,
      hash: tx_hash,
      publicKey:
        "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529f" +
        "a2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
      r: "0x4fbb4e67c0e374d2bbec668f38a6f8bc16583209c2e3154291f53b9e071ab4e4",
      s: "0x3455ef321dd9641dcf52261e9255019ecf816341f6936bf1397aea45b37e4945",
      v: "0xa25",
    });

    await createAndFinalizeBlock(context.polkadotApi);

    const processed_transaction = (
      await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])
    ).result;
    expect(processed_transaction).to.include({
      blockNumber: "0x1",
      hash: tx_hash,
      publicKey:
        "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529f" +
        "a2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
      r: "0x4fbb4e67c0e374d2bbec668f38a6f8bc16583209c2e3154291f53b9e071ab4e4",
      s: "0x3455ef321dd9641dcf52261e9255019ecf816341f6936bf1397aea45b37e4945",
      v: "0xa25",
    });
  });
});
