import { expect } from "chai";
import { Keyring } from "@polkadot/keyring";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import {
  FIRST_CONTRACT_ADDRESS,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_CONTRACT_INCR_ABI,
  TEST_CONTRACT_BYTECODE_INCR,
} from "./constants";

describeWithMoonbeam("Moonbeam RPC (Direct EVM Call)", `simple-specs.json`, (context) => {
  before("create the contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE_INCR,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
  });

  it.only("get transaction by hash", async () => {
    const latestBlock = await context.web3.eth.getBlock("latest");
    expect(latestBlock.transactions.length).to.equal(1);

    const tx_hash = latestBlock.transactions[0];
    const tx = await context.web3.eth.getTransaction(tx_hash);
    console.log("tx", tx);
    console.log("FIRST_CONTRACT_ADDRESS", FIRST_CONTRACT_ADDRESS);
    // @ts-ignore
    expect(tx.creates).to.equal(FIRST_CONTRACT_ADDRESS);
    expect(tx.hash).to.equal(tx_hash);
  });

  it.only("get count", async () => {
    let res = await context.polkadotApi.query.evm.accountStorages(
      FIRST_CONTRACT_ADDRESS,
      "0x0000000000000000000000000000000000000000000000000000000000000000"
    );
    console.log("res", res.toHex(), Number(res));
    expect(Number(res)).to.eq(0);
  });

  it.only("should return contract method result", async function () {
    this.timeout(20000);
    const keyring = new Keyring({ type: "ethereum" });
    const testAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    const contract = new context.web3.eth.Contract(TEST_CONTRACT_INCR_ABI, FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
      gasPrice: "0x01",
    });
    let methodCallBytes: string = contract.methods.incr().encodeABI();
    console.log("methodCallBytes", methodCallBytes);
    let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "latest");
    console.log("nonce", nonce);

    // let hash = await context.polkadotApi.tx.evm
    //   .call(
    //     GENESIS_ACCOUNT,
    //     FIRST_CONTRACT_ADDRESS,
    //     methodCallBytes,
    //     "0x00",
    //     "0x100000",
    //     "0x01",
    //     nonce
    //   )
    //   .signAndSend(testAccount);
    //await new Promise<void>(async (res) => {
    let unsub = await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.evm.call(
          GENESIS_ACCOUNT,
          FIRST_CONTRACT_ADDRESS,
          methodCallBytes,
          "0x00",
          "0x100000",
          "0x01",
          nonce
        )
      )
      .signAndSend(testAccount, { nonce: nonce, era: 0 }, (result) => {
        console.log(`Current registration status is ${result.status}`);
        if (result.status.isInBlock) {
          console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
          unsub();
          //res();
        } else if (result.status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
          unsub();
          // res();
        }
      });
    //});

    //console.log("tx call hash", hash);
    await createAndFinalizeBlock(context.polkadotApi);
    await new Promise((res) => setTimeout(res, 10000));
    let res = await context.polkadotApi.query.evm.accountStorages(
      FIRST_CONTRACT_ADDRESS,
      "0x0000000000000000000000000000000000000000000000000000000000000000"
    );
    console.log("res", res, Number(res));
    console.log("res from web3", await contract.methods.count().call());
    expect(Number(res)).to.eq(1);
    // console.log("hash", hash.toHex());
    // const latestBlock = await context.web3.eth.getBlock("latest");
    // console.log(latestBlock);
    // const tx_hash = latestBlock.transactions[0];
    // console.log("tx_hash", tx_hash);
    // const tx = await context.web3.eth.getTransaction(tx_hash);
    // console.log("tx", tx);
  });
});
