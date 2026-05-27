import "@moonbeam-network/api-augment";

import {
  beforeAll,
  createViemTransaction,
  deployCreateCompiledContract,
  describeSuite,
  expect,
} from "moonwall";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D010413",
  title: "Precompiles - BLS123_81",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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

        const rawTx = await createViemTransaction(context, {
          to: helper1Address,
          data: encodeFunctionData({ abi: helper1Abi, functionName: "testAll", args: [] }),
        });
        const { result } = await context.createBlock(rawTx);

        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });
        expect(receipt.status).toBe("success");

        const rawTx2 = await createViemTransaction(context, {
          to: helper2Address,
          data: encodeFunctionData({ abi: helper2Abi, functionName: "testAll", args: [] }),
        });
        const { result: result2 } = await context.createBlock(rawTx2);

        const receipt2 = await context
          .viem("public")
          .getTransactionReceipt({ hash: result2?.hash as `0x${string}` });
        expect(receipt2.status).toBe("success");
      },
    });
  },
});
