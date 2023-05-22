import "@moonbeam-network/api-augment";

import { TransactionTypes, beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, deployCreateCompiledContract } from "@moonwall/util";
import { Abi } from "abitype";
import { encode } from "punycode";
import { encodeFunctionData, getContract } from "viem";

describeSuite({
  id: "D0609",
  title: "Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let multiplyAddress: `0x${string}`;
    let multiplyAbi: Abi;
    let deployHash: `0x${string}`;
    let multiplyContract: any;

    beforeAll(async function () {
      const { contractAddress, abi, hash, contract } = await deployCreateCompiledContract(
        context,
        "MultiplyBy7"
      );

      multiplyAddress = contractAddress;
      multiplyAbi = abi;
      deployHash = hash;
      multiplyContract = contract;
    });

    // TODO: Re-enable when viem add txntype support for call method
    // for (const txnType of TransactionTypes) {

      it({
        id: "T01",
        title: "should appear in the block transaction list",
        test: async () => {
          const block = await context.viemClient("public").getBlock();
          const txHash = block.transactions[0];
          expect(txHash).toBe(deployHash);
        },
      });
  
      it({
        id: "T02",
        title: "should be in the transaction list",
        test: async () => {
          const tx = await context.viemClient("public").getTransaction({ hash: deployHash });
          expect(tx.hash).to.equal(deployHash);
        },
      });
  
      it({
        id: "T03",
        title: "should provide callable methods",
        test: async function () {
          expect(await multiplyContract.read.multiply([3])).toBe(21n);
        },
      });
  
      it({
        id: "T04",
        title: "should fail for call method with missing parameters",
        test: async function () {
          expect(
            async () =>
              await context.viemClient("public").call({
                account: ALITH_ADDRESS as `0x${string}`,
                to: multiplyAddress,
                data: encodeFunctionData({
                  abi: [{ ...multiplyAbi[0], inputs: [] }] as any,
                  functionName: "multiply",
                  args: []
                }),
              }),
            "Execution succeeded but should have failed"
          ).rejects.toThrowError("revert Contract does not have fallback nor receive functions");
        },
      });
  
      it({
        id: "T05",
        title: "should fail for too many parameters",
        test: async function () {
          expect(
            async () =>
              await context.viemClient("public").call({
                account: ALITH_ADDRESS as `0x${string}`,
                to: multiplyAddress,
                data: encodeFunctionData({
                  abi: [
                    {
                      ...multiplyAbi[0],
                      inputs: [
                        { internalType: "uint256", name: "a", type: "uint256" },
                        { internalType: "uint256", name: "b", type: "uint256" },
                      ],
                    },
                  ] as any,
                  functionName: "multiply",
                  args: [3, 4],
                }),
              }),
            "Execution succeeded but should have failed"
          ).rejects.toThrowError("revert Contract does not have fallback nor receive functions");
        },
      });
  
      it({
        id: "T06",
        title: "should fail for invalid parameters",
        test: async function () {
          expect(
            async () =>
              await context.viemClient("public").call({
                account: ALITH_ADDRESS as `0x${string}`,
                to: multiplyAddress,
                data: encodeFunctionData({
                  abi: [
                    {
                      ...multiplyAbi[0],
                      inputs: [
                        {
                          internalType: "address",
                          name: "a",
                          type: "address",
                        },
                      ],
                    },
                  ] as any,
                  functionName: "multiply",
                  args: ["0x0123456789012345678901234567890123456789"],
                }),
              }),
            "Execution succeeded but should have failed"
          ).rejects.toThrowError("revert Contract does not have fallback nor receive functions");
        },
      });
    // }

   
  },
});
