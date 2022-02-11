import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../util/setup-dev-tests";
import { ALITH, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { createContract } from "../util/transactions";

const BS_TRACER = require("../util/tracer/blockscout_tracer.min.json");
const BS_TRACER_V2 = require("../util/tracer/blockscout_tracer_v2.min.json");

async function createContracts(context) {
  let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
  const { contract: callee, rawTx: rawTx1 } = await createContract(
    context,
    "Callee",
    { nonce: nonce++ },
    []
  );

  const { contract: caller, rawTx: rawTx2 } = await createContract(
    context,
    "Caller",
    { nonce: nonce++ },
    []
  );
  await context.createBlock({
    transactions: [rawTx1, rawTx2],
  });

  return {
    caller: caller,
    calleeAddr: callee.options.address,
    callerAddr: caller.options.address,
    nonce: nonce,
  };
}

async function nestedCall(context, caller, callerAddr, calleeAddr, nonce) {
  let callTx = await context.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      to: callerAddr,
      gas: "0x100000",
      value: "0x00",
      data: caller.methods.someAction(calleeAddr, 6).encodeABI(), // calls callee
      nonce: nonce,
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  return await customWeb3Request(context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]);
}

async function nestedSingle(context) {
  const contracts = await createContracts(context);
  return await nestedCall(
    context,
    contracts.caller,
    contracts.callerAddr,
    contracts.calleeAddr,
    contracts.nonce
  );
}

describeDevMoonbeam(
  "Trace",
  (context) => {
    // This test proves that Raw traces are now stored outside the runtime.
    //
    // Previously exhausted Wasm memory allocation:
    // Thread 'tokio-runtime-worker' panicked at 'Failed to allocate memory:
    // "Allocator ran out of space"'.
    // TODO: raw tracing is temporary disabled
    it.skip("should not overflow Wasm memory", async function () {
      this.timeout(15000);
      const { contract, rawTx } = await createContract(context, "OverflowingTrace", {}, [false]);
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

    // TODO: raw tracing is temporary disabled
    it.skip("should replay over an intermediate state", async function () {
      const { contract, rawTx } = await createContract(context, "Incrementer", {}, [false]);
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

        expect(evmResult).to.equal(target);
      }
    });

    // TODO: raw tracing is temporary disabled
    it.skip("should trace nested contract calls", async function () {
      const send = await nestedSingle(context);
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
      const send = await nestedSingle(context);
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
      const send = await nestedSingle(context);
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
  },
  "Legacy",
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Trace blockscout v2",
  (context) => {
    it("should format as request (Blockscout v2)", async function () {
      const send = await nestedSingle(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        send.result,
        { tracer: BS_TRACER_V2.body },
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
  },
  true
);

describeDevMoonbeamAllEthTxTypes("Trace (Blockscout v2)", (context) => {
  it("should trace correctly out of gas transaction execution (Blockscout v2)", async function () {
    this.timeout(10000);

    const { contract, rawTx } = await createContract(context, "InfiniteContract");
    await context.createBlock({ transactions: [rawTx] });

    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: contract.options.address,
        gas: "0x100000",
        value: "0x00",
        data: "0x5bec9e67",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      callTx.rawTransaction,
    ]);
    await context.createBlock();
    let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
      data.result,
      { tracer: BS_TRACER_V2.body },
    ]);

    expect(trace.result.length).to.be.eq(1);
    expect(trace.result[0].error).to.be.equal("out of gas");
  });

  it("should trace correctly precompiles (Blockscout v2)", async function () {
    this.timeout(10000);

    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: "0x0000000000000000000000000000000000000801",
        gas: "0xdb3b",
        value: "0x0",
        data: "0x4e71d92d",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      callTx.rawTransaction,
    ]);
    await context.createBlock();
    let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
      data.result,
      { tracer: BS_TRACER_V2.body },
    ]);

    expect(trace.result.length).to.be.eq(1);
  });
});

describeDevMoonbeam("Trace", (context) => {
  it("should trace correctly out of gas transaction execution (Blockscout)", async function () {
    this.timeout(10000);

    const { contract, rawTx } = await createContract(context, "InfiniteContract");
    await context.createBlock({ transactions: [rawTx] });

    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: contract.options.address,
        gas: "0x100000",
        value: "0x00",
        data: "0x5bec9e67",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      callTx.rawTransaction,
    ]);
    await context.createBlock();
    let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
      data.result,
      { tracer: BS_TRACER.body },
    ]);

    expect(trace.result.length).to.be.eq(1);
    expect(trace.result[0].error).to.be.equal("out of gas");
  });

  it("should trace correctly precompiles (Blockscout)", async function () {
    this.timeout(10000);

    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: "0x0000000000000000000000000000000000000801",
        gas: "0xdb3b",
        value: "0x0",
        data: "0x4e71d92d",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      callTx.rawTransaction,
    ]);
    await context.createBlock();
    let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
      data.result,
      { tracer: BS_TRACER.body },
    ]);

    expect(trace.result.length).to.be.eq(1);
  });

  it("should trace correctly transfers (raw)", async function () {
    this.timeout(10000);

    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        // arbitrary (non-contract) address to transfer to
        to: ALITH,
        gas: "0xdb3b",
        value: "0x10000000",
        data: "0x",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      callTx.rawTransaction,
    ]);
    await context.createBlock();
    let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [data.result]);

    expect(trace.result.gas).to.be.eq("0x5208"); // 21_000 gas for a transfer.
  });
});

describeDevMoonbeam("Trace", (context) => {
  it("should format as request (callTrace Call)", async function () {
    const send = await nestedSingle(context);
    await context.createBlock();
    let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
      send.result,
      { tracer: "callTracer" },
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
      context,
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
      { tracer: "callTracer" },
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

  it("should trace block by number and hash (callTrace)", async function () {
    const contracts = await createContracts(context);
    let nonce = contracts.nonce;
    await nestedCall(
      context,
      contracts.caller,
      contracts.callerAddr,
      contracts.calleeAddr,
      nonce++
    );
    await nestedCall(
      context,
      contracts.caller,
      contracts.callerAddr,
      contracts.calleeAddr,
      nonce++
    );
    await nestedCall(
      context,
      contracts.caller,
      contracts.callerAddr,
      contracts.calleeAddr,
      nonce++
    );
    await context.createBlock();
    const block = await context.web3.eth.getBlock("latest");
    const block_number = context.web3.utils.toHex(await context.web3.eth.getBlockNumber());
    const block_hash = block.hash;
    // Trace block by number.
    let traceTx = await customWeb3Request(context.web3, "debug_traceBlockByNumber", [
      block_number,
      { tracer: "callTracer" },
    ]);
    expect(block.transactions.length).to.be.equal(traceTx.result.length);
    traceTx.result.forEach((trace) => {
      expect(trace.calls.length).to.be.equal(1);
      expect(Object.keys(trace)).to.deep.equal([
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
    });
    // Trace block by hash (actually the rpc method is an alias of debug_traceBlockByNumber).
    traceTx = await customWeb3Request(context.web3, "debug_traceBlockByHash", [
      block_hash,
      { tracer: "callTracer" },
    ]);
    expect(block.transactions.length).to.be.equal(traceTx.result.length);
    traceTx.result.forEach((trace) => {
      expect(trace.calls.length).to.be.equal(1);
      expect(Object.keys(trace)).to.deep.equal([
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
    });
  });
});
