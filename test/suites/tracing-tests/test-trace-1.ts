import {
  beforeAll,
  customDevRpcRequest,
  describeSuite,
  expect,
  DevModeContext,
  deployCreateCompiledContract,
} from "@moonwall/cli";

import { alith, ALITH_PRIVATE_KEY, createEthersTransaction } from "@moonwall/util";

import { Abi, encodeFunctionData } from "viem";

const BS_TRACER = require("../../helpers/tracer/blockscout_tracer.min.json");

export async function createContracts(context: DevModeContext) {
  let nonce = await context.viem().getTransactionCount({ address: alith.address as `0x${string}` });
  const { contractAddress: callee, abi: abiCallee } = await deployCreateCompiledContract(
    context,
    "TraceCallee",
    { nonce: nonce++ }
  );

  const { contractAddress: caller, abi: abiCaller } = await deployCreateCompiledContract(
    context,
    "TraceCaller",
    { nonce: nonce++ }
  );
  await context.createBlock();

  return {
    abiCallee,
    abiCaller,
    calleeAddr: callee,
    callerAddr: caller,
    nonce: nonce,
  };
}

export async function nestedCall(
  context: DevModeContext,
  callerAddr: string,
  calleeAddr: string,
  abiCaller: Abi,
  nonce: number
) {
  const callTx = await createEthersTransaction(context, {
    to: callerAddr,
    data: encodeFunctionData({
      abi: abiCaller,
      functionName: "someAction",
      args: [calleeAddr, 6],
    }),
    nonce: nonce,
    gasLimit: "0x100000",
    value: "0x00",
  });
  return await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
}

export async function nestedSingle(context: DevModeContext) {
  const contracts = await createContracts(context);
  return await nestedCall(
    context,
    contracts.callerAddr,
    contracts.calleeAddr,
    contracts.abiCaller,
    contracts.nonce
  );
}

describeSuite({
  id: "D3601",
  title: "Trace",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {});

    // This test proves that Raw traces are now stored outside the runtime.
    //
    // Previously exhausted Wasm memory allocation:
    // Thread 'tokio-runtime-worker' panicked at 'Failed to allocate memory:
    // "Allocator ran out of space"'.
    it({
      id: "T01",
      title: "should prevent Wasm memory overflow",
      test: async function () {
        const { abi: abiTraceFilter, hash: hash1 } = await context.deployContract!("TraceFilter", {
          args: [false],
        });
        let receipt = await context.viem().getTransactionReceipt({ hash: hash1 });
        let nonce = await context
          .viem()
          .getTransactionCount({ address: alith.address as `0x${string}` });
        // Produce a +58,000 step trace.
        const callTx = await createEthersTransaction(context, {
          from: alith.address,
          to: receipt.contractAddress,
          data: encodeFunctionData({
            abi: abiTraceFilter,
            functionName: "set_and_loop",
            args: [10],
          }),
          nonce: nonce,
          gasLimit: "0x100000",
          privateKey: ALITH_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(callTx);

        expect(
          async () => await customDevRpcRequest("debug_traceTransaction", [result?.hash]),
          "Trace should be reverted but it worked instead"
        ).rejects.toThrowError(
          "replayed transaction generated too much data. try disabling memory or storage?"
        );
      },
    });

    it({
      id: "T02",
      title: "should replay over an intermediate state",
      test: async function () {
        const { abi: abiIncrementor, hash: hash1 } = await context.deployContract!("Incrementor");
        let receipt = await context.viem().getTransactionReceipt({ hash: hash1 });

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
        let nonce = await context
          .viem()
          .getTransactionCount({ address: alith.address as `0x${string}` });

        // Create 10 transactions in a block.
        for (let numTxs = nonce; numTxs <= nonce + totalTxs; numTxs++) {
          const callTx = await createEthersTransaction(context, {
            from: alith.address,
            to: receipt.contractAddress,
            data: encodeFunctionData({
              abi: abiIncrementor,
              functionName: "incr",
              args: [1],
            }),
            nonce: numTxs,
            gasLimit: "0x100000",
            privateKey: ALITH_PRIVATE_KEY,
          });

          const data = await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
          //console.log(data)
          txs.push(data);
        }
        await context.createBlock();

        // Trace 5 target transactions on it.
        for (let target of targets) {
          let index = target - 1;

          await context.viem().getTransactionReceipt({ hash: txs[index] });

          let intermediateTx = await customDevRpcRequest("debug_traceTransaction", [txs[index]]);

          let evmResult = context.web3().utils.hexToNumber("0x" + intermediateTx.returnValue);
          expect(evmResult).to.equal(target);
        }
      },
    });

    it({
      id: "T03",
      title: "should trace nested contract calls",
      test: async function () {
        const send = await nestedSingle(context);
        await context.createBlock();
        let traceTx = await customDevRpcRequest("debug_traceTransaction", [send]);
        let logs = [];
        for (let log of traceTx.structLogs) {
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
      },
    });

    it({
      id: "T04",
      title: "should use optional disable parameters",
      test: async function () {
        const send = await nestedSingle(context);
        await context.createBlock();
        let traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { disableMemory: true, disableStack: true, disableStorage: true },
        ]);
        let logs = [];
        for (let log of traceTx.structLogs) {
          if (
            log.hasOwnProperty("storage") ||
            log.hasOwnProperty("memory") ||
            log.hasOwnProperty("stack")
          ) {
            logs.push(log);
          }
        }
        expect(logs.length).to.be.equal(0);
      },
    });

    it({
      id: "T05",
      title: "should format as request (Blockscout)",
      test: async function () {
        const send = await nestedSingle(context);
        await context.createBlock();
        let traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { tracer: BS_TRACER.body },
        ]);
        let entries = traceTx;
        expect(entries).to.be.lengthOf(2);
        let resCaller = entries[0];
        let resCallee = entries[1];
        expect(resCaller.callType).to.be.equal("call");
        expect(resCallee.type).to.be.equal("call");
        expect(resCallee.from).to.be.equal(resCaller.to);
        expect(resCaller.traceAddress).to.be.empty;
        expect(resCallee.traceAddress.length).to.be.eq(1);
        expect(resCallee.traceAddress[0]).to.be.eq(0);
      },
    });

    it({
      id: "T06",
      title: "should format as request (Blockscout)",
      test: async function () {
        const send = await nestedSingle(context);
        await context.createBlock();
        let traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { tracer: BS_TRACER.body },
        ]);
        let entries = traceTx;
        expect(entries).to.be.lengthOf(2);
        let resCaller = entries[0];
        let resCallee = entries[1];
        expect(resCaller.callType).to.be.equal("call");
        expect(resCallee.type).to.be.equal("call");
        expect(resCallee.from).to.be.equal(resCaller.to);
        expect(resCaller.traceAddress).to.be.empty;
        expect(resCallee.traceAddress.length).to.be.eq(1);
        expect(resCallee.traceAddress[0]).to.be.eq(0);
      },
    });
  },
});
