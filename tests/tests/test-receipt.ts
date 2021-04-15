import { expect } from "chai";
import { Keyring } from "@polkadot/keyring";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

const INCREMENTER = require("./constants/IncrementerWithEvent.json");

const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GENESIS_ACCOUNT_PRIVATE_KEY =
  "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

describeWithMoonbeam("Moonbeam RPC (Receipt)", `simple-specs.json`, (context) => {
  step("Receipt and events logs should contain valid values", async function () {
    const createTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: INCREMENTER.bytecode,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    let send = await customRequest(context.web3, "eth_sendRawTransaction", [
      createTx.rawTransaction,
    ]);
    await createAndFinalizeBlock(context.polkadotApi);
    let receipt = await context.web3.eth.getTransactionReceipt(send.result);
    const contractAddress = receipt.contractAddress;
    const contract = new context.web3.eth.Contract(INCREMENTER.abi, contractAddress);

    const callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: contractAddress,
        gas: "0x100000",
        value: "0x00",
        data: contract.methods.increment().encodeABI(),
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    send = await customRequest(context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    const block = await context.web3.eth.getBlock(2);
    receipt = await context.web3.eth.getTransactionReceipt(send.result);

    expect(receipt.blockHash).to.be.eq(block.hash);
    expect(receipt.blockNumber).to.be.eq(block.number);
    expect(receipt.from).to.be.eq(GENESIS_ACCOUNT);
    expect(receipt.logs.length).to.be.eq(1);
    expect(receipt.logs[0].address).to.be.eq(contractAddress);
    expect(receipt.logs[0].blockHash).to.be.eq(block.hash);
    expect("0x" + receipt.logs[0].data.substring(26, receipt.logs[0].data.length + 1)).to.be.eq(
      GENESIS_ACCOUNT
    );
    expect(receipt.logs[0].transactionHash).to.be.eq(send.result);
  });
});
