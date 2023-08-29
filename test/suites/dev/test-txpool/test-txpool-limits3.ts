import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  createRawTransfer,
  sendRawTransaction,
} from "@moonwall/util";

describeSuite({
  id: "D3305",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    // 8192 is the number of tx that can be sent to the Pool
    // before it throws an error and drops all tx
    it({
      id: "T01",
      title:
        "should be able to send 8192 tx to the pool " +
        "and have them all published within the following blocks",
      test: async function () {
        for (let i = 0; i < 8192; i++) {
          // for (let i = 0; i < 8192; i++) {
          const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
            nonce: i,
            gas: 400000n,
          });
          await sendRawTransaction(context, rawTxn);
        }
        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;

        expect(txPoolSize).toBe(8192);

        let blocks = 1;
        while (true) {
          await context.createBlock();

          const inspectBlob = (await context
            .viem()
            .transport.request({ method: "txpool_inspect" })) as any;
          const txPoolSize = Object.keys(
            inspectBlob.pending[ALITH_ADDRESS.toLowerCase()] || {}
          ).length;
          log(`Transactions left in pool: ${txPoolSize}`);

          if ((await context.viem().getBlock()).transactions.length == 0) {
            break;
          }
          blocks++;
        }
        log(`Transaction pool was emptied in ${blocks} blocks.`);
      },
    });
  },
});
