import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { createContract } from "../util/transactions";

const BS_TRACER = require("../util/tracer/blockscout_tracer.min.json");

async function nested(context) {
  let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
  const { contract: callee, rawTx: rawTx1 } = await createContract(
    context.web3,
    "Callee",
    { nonce: nonce++ },
    []
  );

  const { contract: caller, rawTx: rawTx2 } = await createContract(
    context.web3,
    "Caller",
    { nonce: nonce++ },
    []
  );
  await context.createBlock({
    transactions: [rawTx1, rawTx2],
  });

  const calleeAddr = callee.options.address;
  const callerAddr = caller.options.address;

  // Nested call
  let callTx = await context.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      to: callerAddr,
      gas: "0x100000",
      value: "0x00",
      data: caller.methods.someAction(calleeAddr, 6).encodeABI(), // calls callee
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  return await customWeb3Request(context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]);
}

describeDevMoonbeam(
  "Trace",
  (context) => {
    // This test proves that Raw traces are now stored outside the runtime.
    //
    // Previously exhausted Wasm memory allocation:
    // Thread 'tokio-runtime-worker' panicked at 'Failed to allocate memory:
    // "Allocator ran out of space"'.
    it("should not overflow Wasm memory", async function () {
      this.timeout(15000);
      const { contract, rawTx } = await createContract(context.web3, "OverflowingTrace", {}, [
        false,
      ]);
      const { txResults } = await context.createBlock({
        transactions: [rawTx],
      });
      let receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);
      let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      // Produce a +58,000 step trace.
      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          to: receipt.contractAddress,
          gas: "0x100000",
          value: "0x00",
          nonce: nonce,
          data: contract.methods.set_and_loop(10).encodeABI(),
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        callTx.rawTransaction,
      ]);
      await context.createBlock();
      let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [data.result]);
      expect(trace.result.stepLogs.length).to.equal(58219);
    });

    it("should replay over an intermediate state", async function () {
      const { contract, rawTx } = await createContract(context.web3, "Incrementer", {}, [false]);
      const { txResults } = await context.createBlock({
        transactions: [rawTx],
      });
      let receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);

      // In our case, the total number of transactions == the max value of the incrementer.
      // If we trace the last transaction of the block, should return the total number of
      // transactions we executed (10).
      // If we trace the 5th transaction, should return 5 and so on.
      //
      // So we set 5 different target txs for a single block: the 1st, 3 intermediate, and
      // the last.
      const totalTxs = 10;
      let targets = [1, 2, 5, 8, 10];
      let txs = [];
      let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      // Create 10 transactions in a block.
      for (let numTxs = nonce; numTxs <= nonce + totalTxs; numTxs++) {
        let callTx = await context.web3.eth.accounts.signTransaction(
          {
            from: GENESIS_ACCOUNT,
            to: receipt.contractAddress,
            gas: "0x100000",
            value: "0x00",
            nonce: numTxs,
            data: contract.methods.sum(1).encodeABI(), // increments by one
          },
          GENESIS_ACCOUNT_PRIVATE_KEY
        );

        const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
          callTx.rawTransaction,
        ]);
        txs.push(data.result);
      }
      await context.createBlock();
      // Trace 5 target transactions on it.
      for (let target of targets) {
        let index = target - 1;

        await context.web3.eth.getTransactionReceipt(txs[index]);

        let intermediateTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
          txs[index],
        ]);

        let evmResult = context.web3.utils.hexToNumber("0x" + intermediateTx.result.returnValue);

        // console.log(`Matching target ${target} against evm result ${evm_result}`);
        expect(evmResult).to.equal(target);
      }
    });

    it("should trace nested contract calls", async function () {
      const send = await nested(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [send.result]);
      let logs = [];
      for (let log of traceTx.result.stepLogs) {
        if (logs.length == 1) {
          logs.push(log);
        }
        if (log.op == "RETURN") {
          logs.push(log);
        }
      }
      expect(logs).to.be.lengthOf(2);
      expect(logs[0].depth).to.be.equal(2);
      expect(logs[1].depth).to.be.equal(1);
    });

    it("should use optional disable parameters", async function () {
      const send = await nested(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        send.result,
        { disableMemory: true, disableStack: true, disableStorage: true },
      ]);
      let logs = [];
      for (let log of traceTx.result.stepLogs) {
        if (
          log.hasOwnProperty("storage") ||
          log.hasOwnProperty("memory") ||
          log.hasOwnProperty("stack")
        ) {
          logs.push(log);
        }
      }
      expect(logs.length).to.be.equal(0);
    });

    it("should format as request (Blockscout)", async function () {
      const send = await nested(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        send.result,
        { tracer: BS_TRACER.body },
      ]);
      let entries = traceTx.result;
      expect(entries).to.be.lengthOf(2);
      let resCaller = entries[0];
      let resCallee = entries[1];
      expect(resCaller.callType).to.be.equal("call");
      expect(resCallee.type).to.be.equal("call");
      expect(resCallee.from).to.be.equal(resCaller.to);
      expect(resCaller.traceAddress).to.be.empty;
      expect(resCallee.traceAddress.length).to.be.eq(1);
      expect(resCallee.traceAddress[0]).to.be.eq(0);
    });

    it("should format as request (callTrace Call)", async function () {
      const send = await nested(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        send.result,
        { tracer: "callTrace" },
      ]);
      let res = traceTx.result;
      // Fields
      expect(Object.keys(res)).to.deep.equal([
        "calls",
        "from",
        "gas",
        "gasUsed",
        "input",
        "output",
        "to",
        "type",
        "value",
      ]);
      // Type
      expect(res.type).to.be.equal("CALL");
      // Nested calls
      let calls = res.calls;
      expect(calls.length).to.be.eq(1);
      let nested_call = calls[0];
      expect(res.to).to.be.equal(nested_call.from);
      expect(nested_call.type).to.be.equal("CALL");
    });

    it("should format as request (callTrace Create)", async function () {
      let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      const { contract: callee, rawTx: rawTx1 } = await createContract(
        context.web3,
        "Callee",
        { nonce: nonce++ },
        []
      );

      let { txResults } = await context.createBlock({
        transactions: [rawTx1],
      });
      let createTxHash = txResults[0].result;
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        createTxHash,
        { tracer: "callTrace" },
      ]);

      let res = traceTx.result;
      // Fields
      expect(Object.keys(res)).to.deep.equal([
        "from",
        "gas",
        "gasUsed",
        "input",
        "output",
        "to",
        "type",
        "value",
      ]);
      // Type
      expect(res.type).to.be.equal("CREATE");
    });
  },
  true
);
