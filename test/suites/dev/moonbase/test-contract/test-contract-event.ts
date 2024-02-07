import "@moonbeam-network/api-augment";
import { TransactionTypes, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";

describeSuite({
  id: "D0604",
  title: "Contract event",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: "should contain event",
        test: async function () {
          const { abi, bytecode } = fetchCompiledContract("EventEmitter");

          const rawSigned = await createEthersTransaction(context, {
            data: encodeDeployData({ abi, bytecode, args: [] }),
            txnType,
            gasLimit: 10_000_000,
          });

          const { result } = await context.createBlock(rawSigned);

          expect(result?.successful, "Unsuccessful deploy").toBe(true);
          const receipt = await context
            .viem("public")
            .getTransactionReceipt({ hash: result?.hash as `0x${string}` });

          expect(receipt.logs.length).toBe(1);
          expect(
            "0x" + receipt.logs[0].topics[1]!.substring(26, receipt.logs[0].topics[1]!.length + 1)
          ).toBe(ALITH_ADDRESS.toLowerCase());
        },
      });
    }
  },
});
