import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, createRawTransfer, sendRawTransaction } from "@moonwall/util";
import { getBlockWithRetry } from "../../../../helpers/eth-transactions";

describeSuite({
  id: "D023803",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to fill a block with at least 2300 tx",
      test: async function () {
        for (let i = 0; i < 3000; i++) {
          const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
            nonce: i,
          });
          await sendRawTransaction(context, rawTxn);
        }

        await context.createBlock();
        const block = await getBlockWithRetry(context, { blockNumber: 1n });
        const maxTxnLen = block.transactions.length;
        log(`out ${maxTxnLen}`);
        expect(maxTxnLen).toBeGreaterThan(2300);
      },
    });
  },
});
