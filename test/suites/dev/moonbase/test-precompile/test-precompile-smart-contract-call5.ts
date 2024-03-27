import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D012981",
  title: "Smart Contract Precompile Call - Proxy - Real Account",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let testContractAddress: `0x${string}`;
    let multiplyContractAddress: `0x${string}`;

    beforeAll(async function () {
      const { contractAddress: addr1 } = await context.deployContract!(
        "SmartContractPrecompileCallTest"
      );
      testContractAddress = addr1;

      const { contractAddress: addr3 } = await context.deployContract!("MultiplyBy7");
      multiplyContractAddress = addr3;
    });
    it({
      id: "T01",
      title: "should revert when caller is a smart contract",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractAddress: testContractAddress,
          contractName: "SmartContractPrecompileCallTest",
          functionName: "callBatch",
          gas: 5_000_000n,
          rawTxOnly: true,
          args: [
            multiplyContractAddress,
            [
              encodeFunctionData({
                abi: fetchCompiledContract("MultiplyBy7").abi,
                functionName: "multiply",
                args: [5],
              }),
              encodeFunctionData({
                abi: fetchCompiledContract("MultiplyBy7").abi,
                functionName: "multiply",
                args: [6],
              }),
              encodeFunctionData({
                abi: fetchCompiledContract("MultiplyBy7").abi,
                functionName: "multiply",
                args: [7],
              }),
            ],
          ],
        });

        const { result } = await context.createBlock(rawTxn);

        expectEVMResult(result!.events, "Revert");
        expect(
          async () =>
            await context.writeContract!({
              contractAddress: testContractAddress,
              contractName: "SmartContractPrecompileCallTest",
              functionName: "callBatch",
              args: [
                multiplyContractAddress,
                [
                  encodeFunctionData({
                    abi: fetchCompiledContract("MultiplyBy7").abi,
                    functionName: "multiply",
                    args: [5],
                  }),
                  encodeFunctionData({
                    abi: fetchCompiledContract("MultiplyBy7").abi,
                    functionName: "multiply",
                    args: [6],
                  }),
                  encodeFunctionData({
                    abi: fetchCompiledContract("MultiplyBy7").abi,
                    functionName: "multiply",
                    args: [7],
                  }),
                ],
              ],
            })
        ).rejects.toThrowError("Function not callable by smart contracts");
      },
    });
  },
});
