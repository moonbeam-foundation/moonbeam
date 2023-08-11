import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction, sendRawTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D3308",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title:
        "should be able to send 8192 tx to the pool and have them" +
        " all published within the following blocks - bigger tx",
      timeout: 400_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        for (let i = 0; i < 8192; i++) {
          const rawTxn = await createEthersTransaction(context, {
            data: deployData,
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
