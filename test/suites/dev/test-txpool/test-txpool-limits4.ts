import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, createRawTransfer, sendRawTransaction } from "@moonwall/util";

describeSuite({
  id: "D3306",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "shouldn't work for 8193",
      test: async function () {
        try {
          for (let i = 0; i < 8193; i++) {
            const rawTxn = await createRawTransfer(context, BALTATHAR_ADDRESS, 1n, {
              nonce: i,
              gas: 400000n,
            });
            await sendRawTransaction(context, rawTxn);
          }
        } catch (e: any) {
          expect(e.message).toContain("submit transaction to pool failed: Ok(ImmediatelyDropped)");
        }

        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        expect(inspectBlob).toMatchObject({
          pending: {},
          queued: {},
        });
      },
    });
  },
});
