import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { AbiItem } from "web3-utils";

describeWithMoonbeam("Moonbeam RPC (Precompiles)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  const PRECOMPILED_CONTRACT_ADDRESS = "0000000000000000000000000000000000001000";

  it("Test submitting a transaction that calls precompiled", async function () {
    const RAW_TX = {
      from: GENESIS_ACCOUNT,
      gasPrice: "0x01",
      gas: "0x10000000",
      to: PRECOMPILED_CONTRACT_ADDRESS,
      value: "0x12345",
      data: "0x12345678",
    };

    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(RAW_TX, GENESIS_ACCOUNT_PRIVATE_KEY);

    const tx_res = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.web3);

    let tx_receipt = await customRequest(context.web3, "eth_getTransactionReceipt", [
      tx_res.result,
    ]);
    // console.log(tx_receipt);

    // there's no way to retrieve the return value, so we just verify the tx hash
    expect(tx_receipt.result.transactionHash).equals(
      "0xf5b4d02ae12bd600adfc110cc91461bb19a39c456fbc57884f40752ecab28d3e"
    );
  });

  it.skip("Verify precompiled function eth_call returned value", async function () {
    this.timeout(15000);

    const tx_call = await customRequest(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: "0x01",
        to: "0x0000000000000000000000000000000000001000",
        data: "0x12345678",
      },
    ]);
    expect(tx_call.result).equals("0xdeadbeef12345678");
  });
});
