import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { createEthersTransaction, sendRawTransaction } from "@moonwall/util";
import { Abi, encodeDeployData } from "viem";

describeSuite({
  id: "D4003",
  title: "TxPool - Stroage Growth Limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageLoopAddress: `0x${string}`;
    let storageLoopAbi: Abi;
    it({
      id: "T01",
      title: "should be able to fill a block with 64 tx",
      test: async function () {
        // Each contract creation tx cost 245_586 gas
        // 15_000_000 / 245_586 = 61
        // So, we can fit 61 contract creation tx in a block
        const { abi, bytecode } = fetchCompiledContract("Fibonacci");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        for (let i = 0; i < 120; i++) {
          const rawTxn = await createEthersTransaction(context, {
            data: deployData,
            nonce: i,
            gasLimit: 400000n,
          });
          await sendRawTransaction(context, rawTxn);
        }

        await context.createBlock();
        expect((await context.viem().getBlock()).transactions.length).toBe(61);
      },
    });
  },
});
