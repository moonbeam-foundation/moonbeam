import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D013905",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let deployData: string;
    beforeAll(async () => {
      const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
      deployData = encodeDeployData({
        abi,
        bytecode,
      });
      const txs = await Promise.all(
        new Array(8192).fill(0).map((_, i) =>
          createEthersTransaction(context, {
            data: deployData,
            nonce: i,
          })
        )
      );
      for (const tx of txs) {
        await context.viem().sendRawTransaction({ serializedTransaction: tx });
      }
    });

    it({
      id: "T01",
      title: "should be able to have 8192 tx in the pool",
      timeout: 30_000,
      test: async function () {
        const inspectBlob = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;
        const txPoolSize = Object.keys(inspectBlob.pending[ALITH_ADDRESS.toLowerCase()]).length;
        expect(txPoolSize).toBe(8192);
      },
    });

    it({
      id: "T02",
      title: "should drop the 8193th tx",
      timeout: 30_000,
      test: async function () {
        try {
          await context.viem().sendRawTransaction({
            serializedTransaction: await createEthersTransaction(context, {
              data: deployData,
              nonce: 8192,
            }),
          });
        } catch (e: any) {
          expect(e.message).toContain("submit transaction to pool failed: Ok(ImmediatelyDropped)");
        }
      },
    });

    it({
      id: "T03",
      title: "should be able have them all published within the following blocks",
      timeout: 40_000,
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

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
