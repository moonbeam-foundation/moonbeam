import {
  customDevRpcRequest,
  describeSuite,
  expect,
  deployCreateCompiledContract,
} from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { createContracts, nestedCall, nestedSingle } from "../../helpers";

describeSuite({
  id: "T05",
  title: "Trace (callTrace)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should format as request (Call)",
      test: async function () {
        const send = await nestedSingle(context);
        await context.createBlock();
        const traceTx = await customDevRpcRequest("debug_traceTransaction", [
          send,
          { tracer: "callTracer" },
        ]);
        const res = traceTx;
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
        const calls = res.calls;
        expect(calls.length).to.be.eq(1);
        const nested_call = calls[0];
        expect(res.to).to.be.equal(nested_call.from);
        expect(nested_call.type).to.be.equal("CALL");
      },
    });

    it({
      id: "T02",
      title: "should format as request (Create)",
      test: async function () {
        let nonce = await context
          .viem()
          .getTransactionCount({ address: alith.address as `0x${string}` });
        const { hash: createTxHash } = await deployCreateCompiledContract(context, "TraceCallee", {
          nonce: nonce++,
        });

        const traceTx = await customDevRpcRequest("debug_traceTransaction", [
          createTxHash,
          { tracer: "callTracer" },
        ]);

        // Fields
        expect(Object.keys(traceTx).sort()).to.deep.equal([
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
        expect(traceTx.type).to.be.equal("CREATE");
      },
    });

    it({
      id: "T03",
      title: "should trace block by number and hash",
      test: async function () {
        const contracts = await createContracts(context);
        let nonce = contracts.nonce;
        await nestedCall(
          context,
          contracts.callerAddr,
          contracts.calleeAddr,
          contracts.abiCaller,
          nonce++
        );
        await nestedCall(
          context,
          contracts.callerAddr,
          contracts.calleeAddr,
          contracts.abiCaller,
          nonce++
        );
        await nestedCall(
          context,
          contracts.callerAddr,
          contracts.calleeAddr,
          contracts.abiCaller,
          nonce++
        );
        await context.createBlock();
        const block = await context.viem().getBlock({ blockTag: "latest" });
        const block_number = context.web3().utils.toHex(await context.viem().getBlockNumber());
        const block_hash = block.hash;
        // Trace block by number.
        let traceTx = await customDevRpcRequest("debug_traceBlockByNumber", [
          block_number,
          { tracer: "callTracer" },
        ]);
        expect(block.transactions.length).to.be.equal(traceTx.length);
        traceTx.forEach((trace: { [key: string]: any }) => {
          expect(trace.result.calls.length).to.be.equal(1);
          expect(Object.keys(trace.result).sort()).to.deep.equal([
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
        traceTx = await customDevRpcRequest("debug_traceBlockByHash", [
          block_hash,
          { tracer: "callTracer" },
        ]);
        expect(block.transactions.length).to.be.equal(traceTx.length);
        traceTx.forEach((trace: { [key: string]: any }) => {
          expect(Object.keys(trace.result).sort()).to.deep.equal([
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
      },
    });
  },
});
