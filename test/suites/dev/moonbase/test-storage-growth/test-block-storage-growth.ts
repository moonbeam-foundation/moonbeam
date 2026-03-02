import "@moonbeam-network/api-augment";
import {
  createEthersTransaction,
  describeSuite,
  expect,
  fetchCompiledContract,
  sendRawTransaction,
} from "moonwall";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D023401",
  title: "Storage Block (160Kb) - Storage Growth Limit",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should fill a block with 61 tx at most",
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("Fibonacci");
        const deployData = encodeDeployData({
          abi,
          bytecode,
        });

        for (let i = 0; i < 300; i++) {
          const rawTxn = await createEthersTransaction(context, {
            data: deployData,
            nonce: i,
            gasLimit: 400000n,
          });
          await sendRawTransaction(context, rawTxn);
        }

        const blockNumber = (await context.viem().getBlockNumber()) + 1n;
        await context.createBlock();
        expect((await context.viem().getBlock({ blockNumber })).transactions.length).toBe(264);
      },
    });
  },
});
