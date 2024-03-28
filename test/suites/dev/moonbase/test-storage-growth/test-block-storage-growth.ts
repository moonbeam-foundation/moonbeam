import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { createEthersTransaction, sendRawTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D013501",
  title: "Storage Block (40Kb) - Storage Growth Limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should fill a block with 61 tx at most",
      test: async function () {
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
