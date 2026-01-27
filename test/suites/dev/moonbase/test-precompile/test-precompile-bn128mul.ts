import "@moonbeam-network/api-augment";
import { createViemTransaction, deployCreateCompiledContract, describeSuite } from "moonwall";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D022705",
  title: "Precompiles - bn128mul",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be accessible from a smart contract",
      test: async function () {
        const { abi, contractAddress } = await deployCreateCompiledContract(
          context,
          "HasherChecker"
        );

        const { result } = await context.createBlock(
          createViemTransaction(context, {
            to: contractAddress,
            data: encodeFunctionData({
              abi,
              functionName: "bn128MultiplyCheck",
            }),
          })
        );

        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
