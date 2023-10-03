import { customDevRpcRequest, describeSuite, expect, TransactionTypes } from "@moonwall/cli";

import { nestedSingle } from "./test-trace-1";

const BS_TRACER_V2 = require("../../helpers/tracer/blockscout_tracer_v2.min.json");

describeSuite({
  id: "D3602",
  title: "Trace blockscout v2 - AllEthTxTypes",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: "should format as request",
        test: async function () {
          const send = await nestedSingle(context);
          await context.createBlock();
          let traceTx = await customDevRpcRequest("debug_traceTransaction", [
            send,
            { tracer: BS_TRACER_V2.body },
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
    }
  },
});
