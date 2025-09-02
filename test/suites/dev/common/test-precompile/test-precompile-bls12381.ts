import "@moonbeam-network/api-augment";

import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D012999",
  title: "Precompiles - BLS123_81",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let helper1Address;
    let helper1Abi;

    let helper2Address;
    let helper2Abi;

    beforeAll(async () => {
      const { contractAddress, abi } = await deployCreateCompiledContract(
        context,
        "BLS12381Helper1"
      );
      helper1Address = contractAddress;
      helper1Abi = abi;

      const { contractAddress: contractAddress2, abi: abi2 } = await deployCreateCompiledContract(
        context,
        "BLS12381Helper2"
      );
      helper2Address = contractAddress2;
      helper2Abi = abi2;
    });

    it({
      id: "T01",
      title: "Test all precompiles with test vectors",
      test: async () => {
        // BLS12381.sol contains the test vectors for the precompiles
        // and splits them into two contracts to avoid hitting the gas limit

        const tx = await context.viem().sendTransaction({
          to: helper1Address,
          data: encodeFunctionData({ abi: helper1Abi, functionName: "testAll", args: [] }),
        });
        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash: tx });
        expect(receipt.status).toBe("success");

        const tx2 = await context.viem().sendTransaction({
          to: helper2Address,
          data: encodeFunctionData({ abi: helper2Abi, functionName: "testAll", args: [] }),
        });
        await context.createBlock();

        const receipt2 = await context.viem().getTransactionReceipt({ hash: tx2 });
        expect(receipt2.status).toBe("success");
      },
    });
  },
});
