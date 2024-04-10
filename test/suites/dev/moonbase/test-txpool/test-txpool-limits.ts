import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, createRawTransfer, sendRawTransaction } from "@moonwall/util";

describeSuite({
  id: "D013903",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to fill a block with 2650 tx",
      test: async function () {
        // TODO: test how many transactions can fit in the block
        for (let i = 0; i < 2650; i++) {
          const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
            nonce: i,
          });
          await sendRawTransaction(context, rawTxn);
        }

        await context.createBlock();
        expect((await context.viem().getBlock()).transactions.length).toBe(2650);
      },
    });
  },
});
