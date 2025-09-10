import { customDevRpcRequest, describeSuite, expect, TransactionTypes } from "@moonwall/cli";
import BS_TRACER_V2 from "../../helpers/tracer/blockscout_tracer_v2.min.json";
import { nestedSingle } from "../../helpers";

describeSuite({
  id: "T03",
  title: "Trace blockscout v2 - AllEthTxTypes",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: "should format as request",
        test: async function () {
          const send = await nestedSingle(context);
          await context.createBlock();
          const traceTx = await customDevRpcRequest("debug_traceTransaction", [
            send,
            { tracer: BS_TRACER_V2.body },
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
    }
  },
});
