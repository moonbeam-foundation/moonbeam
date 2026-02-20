import "@moonbeam-network/api-augment";

import { createViemTransaction, deployCreateCompiledContract, describeSuite } from "moonwall";
import { expectEVMResult } from "../../../../helpers";

import { encodeFunctionData } from "viem";

describeSuite({
  id: "D022702",
  title: "Precompiles - blake2",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should be accessible from a smart contract",
      test: async function () {
        const { abi } = await deployCreateCompiledContract(context, "HasherChecker");

        // Execute the contract blake2 call
        const { result } = await context.createBlock(
          createViemTransaction(context, {
            data: encodeFunctionData({
              abi,
              functionName: "blake2Check",
            }),
          })
        );

        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
