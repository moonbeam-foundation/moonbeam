import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

describeWithMoonbeam("Frontier RPC (Constructor Revert)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  // ```
  // pragma solidity >=0.4.22 <0.7.0;
  //
  // contract WillFail {
  //     constructor() public {
  //         require(false);
  //     }
  // }
  // ```
  const FAIL_BYTECODE =
    "6080604052348015600f57600080fd5b506000601a57600080fd5b603f8060276000396000f3fe60806040526000" +
    "80fdfea26469706673582212209f2bb2a4cf155a0e7b26bd34bb01e9b645a92c82e55c5dbdb4b37f8c326edbee64" +
    "736f6c63430006060033";
  const GOOD_BYTECODE =
    "6080604052348015600f57600080fd5b506001601a57600080fd5b603f8060276000396000f3fe60806040526000" +
    "80fdfea2646970667358221220c70bc8b03cdfdf57b5f6c4131b836f9c2c4df01b8202f530555333f2a00e4b8364" +
    "736f6c63430006060033";

  it("should provide a tx receipt after successful deployment", async function () {
    this.timeout(15000);
    const GOOD_TX_HASH = "0x410f72144e2c0e8b48091f5675422b7a8013335cfdb5e83825c5f27cd991ac8c";

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: GOOD_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    expect(
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result: GOOD_TX_HASH,
    });

    // Verify the receipt exists after the block is created
    await createAndFinalizeBlock(context.polkadotApi);
    const receipt = await context.web3.eth.getTransactionReceipt(GOOD_TX_HASH);
    expect(receipt).to.include({
      blockNumber: 1,
      contractAddress: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
      cumulativeGasUsed: 67231,
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gasUsed: 67231,
      to: null,
      transactionHash: GOOD_TX_HASH,
      transactionIndex: 0,
      status: true,
    });
  });

  it("should provide a tx receipt after failed deployment", async function () {
    this.timeout(15000);
    // Transaction hash depends on which nonce we're using. This hash is for nonce 2.
    const FAIL_TX_HASH = "0xe5ba0bd6229c3315cefa16312b6f3674a5f928aed1b7f397596b724fb259c5ba";

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: FAIL_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    expect(
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result: FAIL_TX_HASH,
    });

    await createAndFinalizeBlock(context.polkadotApi);
    const receipt = await context.web3.eth.getTransactionReceipt(FAIL_TX_HASH);
    expect(receipt).to.include({
      blockNumber: 2,
      contractAddress: "0x5c4242beB94dE30b922f57241f1D02f36e906915",
      cumulativeGasUsed: 54600,
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gasUsed: 54600,
      to: null,
      transactionHash: FAIL_TX_HASH,
      transactionIndex: 0,
      status: false,
    });
  });
});
