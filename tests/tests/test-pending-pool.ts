import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_CONTRACT_BYTECODE } from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

describeWithMoonbeam("Frontier RPC (Pending Pool)", `simple-specs.json`, (context) => {
  // Solidity: contract test { function multiply(uint a) public pure returns(uint d)
  // {return a * 7;}}

  it("should return a pending transaction", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
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
      r: "0xe6f6ef2c1072b0e4a6b91f6b8ca408478814611124a54f3bb5c02c039e9541f1",
      s: "0x5c3a49963649c8812de3aa8b84adf77c14e74eea6191a7827e1273158007bac8",
      v: "0xa26",
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
      r: "0xe6f6ef2c1072b0e4a6b91f6b8ca408478814611124a54f3bb5c02c039e9541f1",
      s: "0x5c3a49963649c8812de3aa8b84adf77c14e74eea6191a7827e1273158007bac8",
      v: "0xa26",
    });
  });
});
