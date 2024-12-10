import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D013906",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title:
        "should be able to send 8192 tx to the pool and have them" +
        " all published within the following blocks - bigger tx",
      timeout: 120_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        const txs = await Promise.all(
          new Array(8192).fill(0).map((_, i) =>
            createEthersTransaction(context, {
              data: deployData,
              nonce: i,
              gas: 400000n,
            })
          )
        );
        for (const tx of txs) {
          await context.viem().sendRawTransaction({ serializedTransaction: tx });
        }

        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;

        expect(txPoolSize).toBe(8192);

        let blocks = 1;
        for (;;) {
          await context.createBlock();

          const inspectBlob = (await context
            .viem()
            .transport.request({ method: "txpool_inspect" })) as any;
          const txPoolSize = Object.keys(
            inspectBlob.pending[ALITH_ADDRESS.toLowerCase()] || {}
          ).length;
          log(`Transactions left in pool: ${txPoolSize}`);

          if ((await context.viem().getBlock()).transactions.length === 0) {
            break;
          }
          blocks++;
        }
        log(`Transaction pool was emptied in ${blocks} blocks.`);
      },
    });
  },
});
