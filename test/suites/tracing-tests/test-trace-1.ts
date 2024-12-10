import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_PRIVATE_KEY, alith, createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { nestedSingle } from "../../helpers";
import BS_TRACER from "../../helpers/tracer/blockscout_tracer.min.json" assert { type: "json" };

describeSuite({
  id: "T01",
  title: "Trace",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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
        const receipt = await context.viem().getTransactionReceipt({ hash: hash1 });
        const nonce = await context
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
        const receipt = await context.viem().getTransactionReceipt({ hash: hash1 });

        // In our case, the total number of transactions === the max value of the incrementer.
        // If we trace the last transaction of the block, should return the total number of
        // transactions we executed (10).
        // If we trace the 5th transaction, should return 5 and so on.
        //
        // So we set 5 different target txs for a single block: the 1st, 3 intermediate, and
        // the last.
        const totalTxs = 10;
        const targets = [1, 2, 5, 8, 10];
        const txs: any[] = [];
        const nonce = await context
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
          txs.push(data);
        }
        await context.createBlock();

        // Trace 5 target transactions on it.
        for (const target of targets) {
          const index = target - 1;

          await context.viem().getTransactionReceipt({ hash: txs[index] });

          const intermediateTx = await customDevRpcRequest("debug_traceTransaction", [txs[index]]);

          const evmResult = context.web3().utils.hexToNumber("0x" + intermediateTx.returnValue);
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
        const traceTx = await customDevRpcRequest("debug_traceTransaction", [send]);
        const logs: any[] = [];
        for (const log of traceTx.structLogs) {
          if (logs.length === 1) {
            logs.push(log);
          }
          if (log.op === "RETURN") {
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
        const traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { disableMemory: true, disableStack: true, disableStorage: true },
        ]);
        const logs: any[] = [];
        for (const log of traceTx.structLogs) {
          const hasStorage = Object.prototype.hasOwnProperty.call(log, "storage");
          const hasMemory = Object.prototype.hasOwnProperty.call(log, "memory");
          const hasStack = Object.prototype.hasOwnProperty.call(log, "stack");
          if (hasStorage || hasMemory || hasStack) {
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
        const traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { tracer: BS_TRACER.body },
        ]);
        const entries = traceTx;
        expect(entries).to.be.lengthOf(2);
        const resCaller = entries[0];
        const resCallee = entries[1];
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
        const traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { tracer: BS_TRACER.body },
        ]);
        const entries = traceTx;
        expect(entries).to.be.lengthOf(2);
        const resCaller = entries[0];
        const resCallee = entries[1];
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
