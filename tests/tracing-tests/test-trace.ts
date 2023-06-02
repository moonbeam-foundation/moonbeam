import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { alith, ALITH_PRIVATE_KEY, baltathar } from "../util/accounts";
import { PRECOMPILE_BATCH_ADDRESS, PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS } from "../util/constants";
import { getCompiled } from "../util/contracts";
import { customWeb3Request } from "../util/providers";
import {
  describeDevMoonbeam,
  describeDevMoonbeamAllEthTxTypes,
  DevTestContext,
} from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

const BS_TRACER = require("../util/tracer/blockscout_tracer.min.json");
const BS_TRACER_V2 = require("../util/tracer/blockscout_tracer_v2.min.json");

async function createContracts(context: DevTestContext) {
  let nonce = await context.web3.eth.getTransactionCount(alith.address);
  const { contract: callee, rawTx: rawTx1 } = await createContract(
    context,
    "TraceCallee",
    { nonce: nonce++ },
    []
  );

  const { contract: caller, rawTx: rawTx2 } = await createContract(
    context,
    "TraceCaller",
    { nonce: nonce++ },
    []
  );
  await context.createBlock([rawTx1, rawTx2]);

  return {
    caller: caller,
    calleeAddr: callee.options.address,
    callerAddr: caller.options.address,
    nonce: nonce,
  };
}

async function nestedCall(
  context: DevTestContext,
  caller: Contract,
  callerAddr: string,
  calleeAddr: string,
  nonce: number
) {
  let callTx = await context.web3.eth.accounts.signTransaction(
    {
      from: alith.address,
      to: callerAddr,
      gas: "0x100000",
      value: "0x00",
      data: caller.methods.someAction(calleeAddr, 6).encodeABI(), // calls callee
      nonce: nonce,
    },
    ALITH_PRIVATE_KEY
  );
  return await customWeb3Request(context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]);
}

async function nestedSingle(context: DevTestContext) {
  const contracts = await createContracts(context);
  return await nestedCall(
    context,
    contracts.caller,
    contracts.callerAddr,
    contracts.calleeAddr,
    contracts.nonce
  );
}

// TODO: Refactor dependent tests
describeDevMoonbeam(
  "Trace",
  (context) => {
    // This test proves that Raw traces are now stored outside the runtime.
    //
    // Previously exhausted Wasm memory allocation:
    // Thread 'tokio-runtime-worker' panicked at 'Failed to allocate memory:
    // "Allocator ran out of space"'.
    it("should prevent Wasm memory overflow", async function () {
      const { contract, rawTx } = await createContract(context, "TraceFilter", {}, [false]);
      const {
        result: { hash: hash1 },
      } = await context.createBlock(rawTx);
      let receipt = await context.web3.eth.getTransactionReceipt(hash1);
      let nonce = await context.web3.eth.getTransactionCount(alith.address);
      // Produce a +58,000 step trace.
      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: receipt.contractAddress,
          gas: "0x100000",
          value: "0x00",
          nonce: nonce,
          data: contract.methods.set_and_loop(10).encodeABI(),
        },
        ALITH_PRIVATE_KEY
      );
      const {
        result: { hash: hash2 },
      } = await context.createBlock(callTx.rawTransaction);
      let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [hash2]);
      expect((trace.error as any).message).to.equal(
        "replayed transaction generated too much data. try disabling memory or storage?"
      );
    });

    it("should replay over an intermediate state", async function () {
      const { contract, rawTx } = await createContract(context, "Incrementor");
      const { result } = await context.createBlock(rawTx);
      let receipt = await context.web3.eth.getTransactionReceipt(result.hash);

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
      let nonce = await context.web3.eth.getTransactionCount(alith.address);
      // Create 10 transactions in a block.
      for (let numTxs = nonce; numTxs <= nonce + totalTxs; numTxs++) {
        let callTx = await context.web3.eth.accounts.signTransaction(
          {
            from: alith.address,
            to: receipt.contractAddress,
            gas: "0x100000",
            value: "0x00",
            nonce: numTxs,
            data: contract.methods.incr(1).encodeABI(), // increments by one
          },
          ALITH_PRIVATE_KEY
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

    it("should trace nested contract calls", async function () {
      const send = await nestedSingle(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [send.result]);
      let logs = [];
      for (let log of traceTx.result.structLogs) {
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
      for (let log of traceTx.result.structLogs) {
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
  "moonbase",
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Trace blockscout v2",
  (context) => {
    it("should format as request", async function () {
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

// TODO: Refactor dependent tests
describeDevMoonbeamAllEthTxTypes(
  "Trace (Blockscout v2)",
  (context) => {
    it("should trace correctly out of gas transaction execution", async function () {
      const { contract, rawTx } = await createContract(context, "Looper");
      await context.createBlock(rawTx);

      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: contract.options.address,
          gas: "0x100000",
          value: "0x00",
          data: "0x5bec9e67",
        },
        ALITH_PRIVATE_KEY
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

    it("should trace correctly precompiles", async function () {
      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS,
          gas: "0xdb3b",
          value: "0x0",
          data: "0x4e71d92d",
        },
        ALITH_PRIVATE_KEY
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
  },
  true
);

describeDevMoonbeam(
  "Trace (Blockscout)",
  (context) => {
    it("should trace correctly out of gas transaction execution", async function () {
      const { contract, rawTx } = await createContract(context, "Looper");
      await context.createBlock(rawTx);

      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: contract.options.address,
          gas: "0x100000",
          value: "0x00",
          data: "0x5bec9e67",
        },
        ALITH_PRIVATE_KEY
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

    it("should trace correctly precompiles", async function () {
      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS,
          gas: "0xdb3b",
          value: "0x0",
          data: "0x4e71d92d",
        },
        ALITH_PRIVATE_KEY
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
      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          // arbitrary (non-contract) address to transfer to
          to: baltathar.address,
          gas: "0xdb3b",
          value: "0x10000000",
          data: "0x",
        },
        ALITH_PRIVATE_KEY
      );
      const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        callTx.rawTransaction,
      ]);
      await context.createBlock();
      let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [data.result]);

      expect(trace.result.gas).to.be.eq("0x5208"); // 21_000 gas for a transfer.
    });
  },
  "Legacy",
  "moonbase",
  true
);

describeDevMoonbeam(
  "Trace (callTrace)",
  (context) => {
    it("should format as request (Call)", async function () {
      const send = await nestedSingle(context);
      await context.createBlock();
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        send.result,
        { tracer: "callTracer" },
      ]);
      let res = traceTx.result;
      // Fields
      expect(Object.keys(res).sort()).to.deep.equal([
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

    it("should format as request (Create)", async function () {
      let nonce = await context.web3.eth.getTransactionCount(alith.address);
      const { contract: callee, rawTx: rawTx1 } = await createContract(
        context,
        "TraceCallee",
        { nonce: nonce++ },
        []
      );

      let { result } = await context.createBlock(rawTx1);
      let createTxHash = result.hash;
      let traceTx = await customWeb3Request(context.web3, "debug_traceTransaction", [
        createTxHash,
        { tracer: "callTracer" },
      ]);

      let res = traceTx.result;
      // Fields
      expect(Object.keys(res).sort()).to.deep.equal([
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

    it("should trace block by number and hash", async function () {
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
      traceTx.result.forEach((trace: { [key: string]: any }) => {
        expect(trace.calls.length).to.be.equal(1);
        expect(Object.keys(trace).sort()).to.deep.equal([
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
      traceTx.result.forEach((trace: { [key: string]: any }) => {
        expect(trace.calls.length).to.be.equal(1);
        expect(Object.keys(trace).sort()).to.deep.equal([
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
  },
  "Legacy",
  "moonbase",
  true
);

describeDevMoonbeam(
  "Trace (call list)",
  (context) => {
    it("should correctly trace subcall", async function () {
      const { contract: contractProxy, rawTx } = await createContract(context, "CallForwarder");
      await context.createBlock(rawTx);

      const { contract: contractDummy, rawTx: rawTx2 } = await createContract(
        context,
        "MultiplyBy7"
      );
      await context.createBlock([rawTx2]);

      const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
      const dummyInterface = new ethers.utils.Interface(getCompiled("MultiplyBy7").contract.abi);

      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: contractProxy.options.address,
          gas: "0x100000",
          value: "0x00",
          data: proxyInterface.encodeFunctionData("call", [
            contractDummy.options.address,
            dummyInterface.encodeFunctionData("multiply", [42]),
          ]),
        },
        ALITH_PRIVATE_KEY
      );

      const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        callTx.rawTransaction,
      ]);
      await context.createBlock();
      let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
        data.result,
        { tracer: "callTracer" },
      ]);

      expect(trace.result.from).to.be.eq(alith.address.toLowerCase());
      expect(trace.result.to).to.be.eq(contractProxy.options.address.toLowerCase());
      expect(trace.result.calls.length).to.be.eq(1);
      expect(trace.result.calls[0].from).to.be.eq(contractProxy.options.address.toLowerCase());
      expect(trace.result.calls[0].to).to.be.eq(contractDummy.options.address.toLowerCase());
      expect(trace.result.calls[0].type).to.be.eq("CALL");
    });

    it("should correctly trace delegatecall subcall", async function () {
      const { contract: contractProxy, rawTx } = await createContract(context, "CallForwarder");
      await context.createBlock(rawTx);

      const { contract: contractDummy, rawTx: rawTx2 } = await createContract(
        context,
        "MultiplyBy7"
      );
      await context.createBlock([rawTx2]);

      const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
      const dummyInterface = new ethers.utils.Interface(getCompiled("MultiplyBy7").contract.abi);

      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: contractProxy.options.address,
          gas: "0x100000",
          value: "0x00",
          data: proxyInterface.encodeFunctionData("delegateCall", [
            contractDummy.options.address,
            dummyInterface.encodeFunctionData("multiply", [42]),
          ]),
        },
        ALITH_PRIVATE_KEY
      );

      const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        callTx.rawTransaction,
      ]);
      await context.createBlock();
      let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
        data.result,
        { tracer: "callTracer" },
      ]);

      expect(trace.result.from).to.be.eq(alith.address.toLowerCase());
      expect(trace.result.to).to.be.eq(contractProxy.options.address.toLowerCase());
      expect(trace.result.calls.length).to.be.eq(1);
      expect(trace.result.calls[0].from).to.be.eq(contractProxy.options.address.toLowerCase());
      expect(trace.result.calls[0].to).to.be.eq(contractDummy.options.address.toLowerCase());
      expect(trace.result.calls[0].type).to.be.eq("DELEGATECALL");
    });

    it("should correctly trace precompile subcall (call list)", async function () {
      this.timeout(10000);

      const { contract: contractProxy, rawTx } = await createContract(context, "CallForwarder");
      await context.createBlock(rawTx);

      const { contract: contractDummy, rawTx: rawTx2 } = await createContract(
        context,
        "MultiplyBy7"
      );
      await context.createBlock([rawTx2]);

      const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
      const dummyInterface = new ethers.utils.Interface(getCompiled("MultiplyBy7").contract.abi);
      const batchInterface = new ethers.utils.Interface(
        getCompiled("precompiles/batch/Batch").contract.abi
      );

      let callTx = await context.web3.eth.accounts.signTransaction(
        {
          from: alith.address,
          to: PRECOMPILE_BATCH_ADDRESS,
          gas: "0x100000",
          value: "0x00",
          data: batchInterface.encodeFunctionData("batchAll", [
            [contractProxy.options.address, contractProxy.options.address],
            [],
            [
              proxyInterface.encodeFunctionData("call", [
                contractDummy.options.address,
                dummyInterface.encodeFunctionData("multiply", [42]),
              ]),
              proxyInterface.encodeFunctionData("delegateCall", [
                contractDummy.options.address,
                dummyInterface.encodeFunctionData("multiply", [42]),
              ]),
            ],
            [],
          ]),
        },
        ALITH_PRIVATE_KEY
      );

      const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        callTx.rawTransaction,
      ]);
      await context.createBlock();
      let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
        data.result,
        { tracer: "callTracer" },
      ]);

      expect(trace.result.from).to.be.eq(alith.address.toLowerCase());
      expect(trace.result.to).to.be.eq(PRECOMPILE_BATCH_ADDRESS);
      expect(trace.result.calls.length).to.be.eq(2);

      expect(trace.result.calls[0].from).to.be.eq(PRECOMPILE_BATCH_ADDRESS);
      expect(trace.result.calls[0].to).to.be.eq(contractProxy.options.address.toLowerCase());
      expect(trace.result.calls[0].type).to.be.eq("CALL");

      expect(trace.result.calls[0].calls.length).to.be.eq(1);
      expect(trace.result.calls[0].calls[0].from).to.be.eq(
        contractProxy.options.address.toLowerCase()
      );
      expect(trace.result.calls[0].calls[0].to).to.be.eq(
        contractDummy.options.address.toLowerCase()
      );
      expect(trace.result.calls[0].calls[0].type).to.be.eq("CALL");

      expect(trace.result.calls[1].from).to.be.eq(PRECOMPILE_BATCH_ADDRESS);
      expect(trace.result.calls[1].to).to.be.eq(contractProxy.options.address.toLowerCase());
      expect(trace.result.calls[1].type).to.be.eq("CALL");

      expect(trace.result.calls[1].calls.length).to.be.eq(1);
      expect(trace.result.calls[1].calls[0].from).to.be.eq(
        contractProxy.options.address.toLowerCase()
      );
      expect(trace.result.calls[1].calls[0].to).to.be.eq(
        contractDummy.options.address.toLowerCase()
      );
      expect(trace.result.calls[1].calls[0].type).to.be.eq("DELEGATECALL");
    });
  },
  "Legacy",
  "moonbase",
  true
);

describeDevMoonbeam("Raw trace limits", (context) => {
  it("it should not trace call that would produce too big responses", async function () {
    this.timeout(50000);
    const { contract: contract, rawTx } = await createContract(context, "TraceFilter", {}, [false]);
    await context.createBlock(rawTx);

    const contractInterface = new ethers.utils.Interface(getCompiled("TraceFilter").contract.abi);

    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: contract.options.address,
        gas: "0x800000",
        value: "0x00",
        data: contractInterface.encodeFunctionData("heavy_steps", [
          100, // number of storage modified
          1000, // numbers of simple steps (that will have 100 storage items in trace)
        ]),
      },
      ALITH_PRIVATE_KEY
    );

    const data = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      callTx.rawTransaction,
    ]);
    await context.createBlock();
    let trace = await customWeb3Request(context.web3, "debug_traceTransaction", [data.result]);

    expect(trace.error).to.deep.eq({
      code: -32603,
      message: "replayed transaction generated too much data. try disabling memory or storage?",
    });
  });
});
