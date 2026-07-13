import "@moonbeam-network/api-augment";
import {
  createEthersTransaction,
  describeSuite,
  expect,
  fetchCompiledContract,
  sendRawTransaction,
} from "moonwall";
import { encodeDeployData } from "viem";
import { getBlockWithRetry } from "../../../../helpers/eth-transactions";

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
        // The eth RPC can lag the freshly sealed block; retry until it is
        // available instead of reading it once immediately after sealing.
        expect((await getBlockWithRetry(context, { blockNumber })).transactions.length).toBe(264);
      },
    });
  },
});
