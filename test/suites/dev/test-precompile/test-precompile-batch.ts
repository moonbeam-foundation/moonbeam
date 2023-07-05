import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  PRECOMPILE_BATCH_ADDRESS,
  createViemTransaction,
  sendRawTransaction,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D2520",
  title: "Batch - All functions should consume the same gas",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should consume the same gas",
      test: async function () {
        const { abi: batchInterface } = fetchCompiledContract("Batch");

        // each tx have a different gas limit to ensure it doesn't impact gas used
        const batchAllTx = await createViemTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          gas: 1114112n,
          data: encodeFunctionData({
            abi: batchInterface,
            functionName: "batchAll",
            args: [
              [BALTATHAR_ADDRESS, CHARLETH_ADDRESS],
              ["1000000000000000000", "2000000000000000000"],
              [],
              [],
            ],
          }),
        });

        const batchSomeTx = await createViemTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          gas: 1179648n,
          nonce: 1,
          data: encodeFunctionData({
            abi: batchInterface,
            functionName: "batchSome",
            args: [
              [BALTATHAR_ADDRESS, CHARLETH_ADDRESS],
              ["1000000000000000000", "2000000000000000000"],
              [],
              [],
            ],
          }),
        });

        const batchSomeUntilFailureTx = await createViemTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          gas: 1245184n,
          nonce: 2,
          data: encodeFunctionData({
            abi: batchInterface,
            functionName: "batchSomeUntilFailure",
            args: [
              [BALTATHAR_ADDRESS, CHARLETH_ADDRESS],
              ["1000000000000000000", "2000000000000000000"],
              [],
              [],
            ],
          }),
        });

        const batchAllResult = await sendRawTransaction(context, batchAllTx);
        const batchSomeResult = await sendRawTransaction(context, batchSomeTx);
        const batchSomeUntilFailureResult = await sendRawTransaction(
          context,
          batchSomeUntilFailureTx
        );

        await context.createBlock();

        const batchAllReceipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: batchAllResult as `0x${string}` });
        const batchSomeReceipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: batchSomeResult as `0x${string}` });
        const batchSomeUntilFailureReceipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: batchSomeUntilFailureResult as `0x${string}` });

        expect(batchAllReceipt["gasUsed"]).to.equal(44932n);
        expect(batchSomeReceipt["gasUsed"]).to.equal(44932n);
        expect(batchSomeUntilFailureReceipt["gasUsed"]).to.equal(44932n);
      },
    });
  },
});
