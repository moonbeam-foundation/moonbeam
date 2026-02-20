import "@moonbeam-network/api-augment";
import { createEthersTransaction, describeSuite, expect, fetchCompiledContract } from "moonwall";
import { encodeDeployData } from "viem";
import { getBlockWithRetry } from "../../../../helpers/eth-transactions";

describeSuite({
  id: "D023804",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should be able to fill a block with 70 contract creations tx",
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        const txs = await Promise.all(
          new Array(300).fill(0).map((_, i) =>
            createEthersTransaction(context, {
              data: deployData,
              nonce: i,
              gasLimit: 400000n,
            })
          )
        );
        for (const tx of txs) {
          await context.viem().sendRawTransaction({ serializedTransaction: tx });
        }

        await context.createBlock();
        expect((await getBlockWithRetry(context)).transactions.length).toBe(284);
      },
    });
  },
});
