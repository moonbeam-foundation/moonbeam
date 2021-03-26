import { expect } from "chai";
import { Keyring } from "@polkadot/keyring";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

const INCREMENTER = require("./constants/Incrementer.json");
const CALLEE = require("./constants/Callee.json");
const CALLER = require("./constants/Caller.json");
const BS_TRACER = require("./constants/blockscout_tracer.min.json");

const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GENESIS_ACCOUNT_PRIVATE_KEY =
  "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

async function nested(context) {
  // Create Callee contract.
  const calleeTx = await context.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      data: CALLEE.bytecode,
      value: "0x00",
      gasPrice: "0x01",
      gas: "0x100000",
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  let send = await customRequest(context.web3, "eth_sendRawTransaction", [calleeTx.rawTransaction]);
  await createAndFinalizeBlock(context.polkadotApi);
  let receipt = await context.web3.eth.getTransactionReceipt(send.result);
  const callee_addr = receipt.contractAddress;
  const callee = new context.web3.eth.Contract(CALLEE.abi, callee_addr);
  // Create Caller contract.
  const callerTx = await context.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      data: CALLER.bytecode,
      value: "0x00",
      gasPrice: "0x01",
      gas: "0x100000",
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  send = await customRequest(context.web3, "eth_sendRawTransaction", [callerTx.rawTransaction]);
  await createAndFinalizeBlock(context.polkadotApi);
  receipt = await context.web3.eth.getTransactionReceipt(send.result);
  const caller_addr = receipt.contractAddress;
  const caller = new context.web3.eth.Contract(CALLER.abi, caller_addr);
  // Nested call
  let callTx = await context.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      to: caller_addr,
      gas: "0x100000",
      value: "0x00",
      data: caller.methods.someAction(callee_addr, 6).encodeABI(), // calls callee
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  return await customRequest(context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]);
}

describeWithMoonbeam("Moonbeam RPC (debug_traceTransaction)", `simple-specs.json`, (context) => {
  step("[Raw] should replay over an intermediate state", async function () {
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
    // This contract's `sum` method receives a number as an argument, increments the storage and
    // returns the current value.
    let contract = new context.web3.eth.Contract(INCREMENTER.abi, receipt.contractAddress);

    // In our case, the total number of transactions == the max value of the incrementer.
    // If we trace the last transaction of the block, should return the total number of
    // transactions we executed (10).
    // If we trace the 5th transaction, should return 5 and so on.
    //
    // So we set 5 different target txs for a single block: the 1st, 3 intermediate, and
    // the last.
    const total_txs = 10;
    let targets = [1, 2, 5, 8, 10];
    let iteration = 0;
    let txs = [];
    let num_txs;
    // Create 10 transactions in a block.
    for (num_txs = 1; num_txs <= total_txs; num_txs++) {
      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          to: receipt.contractAddress,
          gas: "0x100000",
          value: "0x00",
          nonce: num_txs,
          data: contract.methods.sum(1).encodeABI(), // increments by one
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      send = await customRequest(context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]);
      txs.push(send.result);
    }
    await createAndFinalizeBlock(context.polkadotApi);
    // Trace 5 target transactions on it.
    for (let target of targets) {
      let index = target - 1;

      let receipt = await context.web3.eth.getTransactionReceipt(txs[index]);

      let intermediate_tx = await customRequest(context.web3, "debug_traceTransaction", [
        txs[index],
      ]);

      let evm_result = context.web3.utils.hexToNumber("0x" + intermediate_tx.result.returnValue);

      // console.log(`Matching target ${target} against evm result ${evm_result}`);
      expect(evm_result).to.equal(target);
    }
  });

  step("[Raw] should trace nested contract calls", async function () {
    const send = await nested(context);
    await createAndFinalizeBlock(context.polkadotApi);
    let traceTx = await customRequest(context.web3, "debug_traceTransaction", [send.result]);
    let logs = [];
    for (let log of traceTx.result.stepLogs) {
      if (logs.length == 1) {
        logs.push(log);
      }
      if (log.op == "RETURN") {
        logs.push(log);
      }
    }
    expect(logs.length).to.be.equal(2);
    expect(logs[0].depth).to.be.equal(2);
    expect(logs[1].depth).to.be.equal(1);
  });

  step("[Blockscout] should trace nested contract calls", async function () {
    const send = await nested(context);
    await createAndFinalizeBlock(context.polkadotApi);
    let traceTx = await customRequest(context.web3, "debug_traceTransaction", [
      send.result,
      { tracer: BS_TRACER.body },
    ]);
    let entries = traceTx.result;
    expect(entries.length).to.be.equal(2);
    let resCaller = entries[0];
    let resCallee = entries[1];
    expect(resCaller.callType).to.be.equal("call");
    expect(resCallee.type).to.be.equal("call");
    expect(resCallee.from).to.be.equal(resCaller.to);
    expect(resCaller.traceAddress).to.be.empty;
    expect(resCallee.traceAddress.length).to.be.eq(1);
    expect(resCallee.traceAddress[0]).to.be.eq(0);
  });
});
