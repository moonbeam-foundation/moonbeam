import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import type { Abi } from "viem";

describeSuite({
  id: "D020101",
  title: "Precompiles - Assets-ERC20 (LocalAssets Removal)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const precompileRegistryAddress = "0x0000000000000000000000000000000000000815";
    const localAssets = [
      "0xffFfFffeFd9d0bf45a2947A519a741c4b9E99EB6",
      "0xFfFFfffeD1b57d12738e41EAe0124E285e5e86a3",
      "0xFFfFfFfeAbB8953aC77edEcE4A6e251A849C7CdF",
      "0xFFfffFFecB45aFD30a637967995394Cc88C0c194",
    ];
    const notLocalAssets = [
      "0xfffffFFeFD9D0bF45A2947A519a741C4B9e99eB7",
      "0xfFFffffEcB45AFd30a637967995394cC88c0C195",
    ];
    let erc20Abi: Abi;
    let precompileRegistryAbi: Abi;

    beforeAll(async () => {
      const { abi } = await deployCreateCompiledContract(context, "ERC20Instance");
      erc20Abi = abi;
      const { abi: _precompileRegistryAbi } = await deployCreateCompiledContract(
        context,
        "PrecompileRegistry"
      );
      precompileRegistryAbi = _precompileRegistryAbi;
    });

    it({
      id: "T01",
      title: "ensure evm calls on local xc-20 addresses get reverted",
      test: async () => {
        const addresses = [
          ...localAssets.map((address) => ({ address, local: true })),
          ...notLocalAssets.map((address) => ({ address, local: false })),
        ];

        // Call every address and ensure it only local asset addresses get reverted
        for (const { address, local } of addresses) {
          let revertReason = "";
          try {
            await context.viem().readContract({
              address: address as `0x${string}`,
              abi: erc20Abi,
              functionName: "balanceOf",
              args: [ALITH_ADDRESS],
            });
          } catch (e) {
            revertReason = e.cause.reason;
          }

          if (local) {
            expect(revertReason).toBe("Removed precompile");
          } else {
            expect(revertReason).to.be.undefined;
          }
        }
      },
    });

    it({
      id: "T02",
      title: "Local assets should be inactive in the precompile registry",
      test: async () => {
        const addresses = [
          ...localAssets.map((address) => ({ address, local: true })),
          ...notLocalAssets.map((address) => ({ address, local: false })),
        ];

        for (const { address, local } of addresses) {
          const isPrecompileResult = await context.viem().readContract({
            address: precompileRegistryAddress,
            abi: precompileRegistryAbi,
            functionName: "isPrecompile",
            args: [address],
          });
          const isActivePrecompileResult = await context.viem().readContract({
            address: precompileRegistryAddress,
            abi: precompileRegistryAbi,
            functionName: "isActivePrecompile",
            args: [address],
          });

          if (local) {
            // All local asset addresses are still considereded precompiles
            expect(isPrecompileResult).to.be.true;
            // All local asset precompiles should be inactive
            expect(isActivePrecompileResult).to.be.false;
          } else {
            expect(isPrecompileResult).to.be.false;
          }
        }
      },
    });
  },
});
