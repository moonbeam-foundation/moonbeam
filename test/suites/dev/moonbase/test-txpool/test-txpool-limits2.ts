import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D013904",
  title: "TxPool - Limits",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to fill a block with 141 contract creations tx",
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
        expect((await context.viem().getBlock()).transactions.length).toBe(141);
      },
    });
  },
});
