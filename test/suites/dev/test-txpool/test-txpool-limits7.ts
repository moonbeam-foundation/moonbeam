import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { createEthersTransaction, sendRawTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D3309",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "shouldn't work for 8193 - bigger tx",
      timeout: 400_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });
        try {
          for (let i = 0; i < 8192; i++) {
            const rawTxn = await createEthersTransaction(context, {
              data: deployData,
              nonce: i,
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
