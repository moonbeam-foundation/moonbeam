import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  createAndFinalizeBlock,
  createRawTransfer,
  sendRawTransaction,
} from "@moonwall/util";

describeSuite({
  id: "D013903",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to fill a block with 582 tx",
      test: async function () {
        // TODO: test how many transactions can fit in the block
        for (let i = 0; i < 600; i++) {
          const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, i + 1, {
            nonce: i,
          });
          await sendRawTransaction(context, rawTxn);
        }
        console.log((await context.polkadotJs().rpc.author.pendingExtrinsics()).length);

        console.log(await createAndFinalizeBlock(context.polkadotJs(), undefined, true));
        //await context.createBlock();
        expect((await context.viem().getBlock()).transactions.length).toBe(568);
      },
    });
  },
});
