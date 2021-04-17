import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_ACCOUNT } from "./constants";
import { Keyring } from "@polkadot/keyring";

describeWithMoonbeam("Frontier RPC (Constructor Revert)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
  const TEST_ACCOUNT_PRIVATE_KEY =
    "0x1111111111111111111111111111111111111111111111111111111111111111";
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

  before(async function () {
    // We send some money to TEST_ACCOUNT in order to have deterministic nonces
    const testAccount1 = context.web3.eth.accounts.privateKeyToAccount(TEST_ACCOUNT_PRIVATE_KEY);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: testAccount1.address,
        value: "0x200000",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
  });

  it("should provide a tx receipt after successful deployment", async function () {
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

    const goodTxHash = context.web3.utils.keccak256(tx.rawTransaction);

    expect(
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result: tx.transactionHash,
    });

    // Verify the receipt exists after the block is created
    await createAndFinalizeBlock(context.polkadotApi);
    let currentHeight = await context.web3.eth.getBlockNumber();

    const receipt = await context.web3.eth.getTransactionReceipt(goodTxHash);
    expect(receipt).to.include({
      blockNumber: currentHeight,
      contractAddress: "0x5c4242beB94dE30b922f57241f1D02f36e906915",
      cumulativeGasUsed: 67231,
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gasUsed: 67231,
      to: null,
      transactionHash: goodTxHash,
      transactionIndex: 0,
      status: true,
    });
  });

  it("should provide a tx receipt after failed deployment", async function () {
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: FAIL_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      TEST_ACCOUNT_PRIVATE_KEY
    );

    const failTxHash = context.web3.utils.keccak256(tx.rawTransaction);

    expect(
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result: tx.transactionHash,
    });

    await createAndFinalizeBlock(context.polkadotApi);
    let current_height = await context.web3.eth.getBlockNumber();

    const receipt = await context.web3.eth.getTransactionReceipt(failTxHash);
    expect(receipt).to.include({
      blockNumber: current_height,
      contractAddress: "0xAE519FC2Ba8e6fFE6473195c092bF1BAe986ff90",
      cumulativeGasUsed: 54600,
      from: "0x19e7e376e7c213b7e7e7e46cc70a5dd086daff2a",
      gasUsed: 54600,
      to: null,
      transactionHash: failTxHash,
      transactionIndex: 0,
      status: false,
    });
  });
});
