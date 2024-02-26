import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import { extractRevertReason } from "../../../../helpers";

describeSuite({
  id: "D012901",
  title: "Precompiles - Assets-ERC20 (LocalAssets Removal)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const precompileRegistryAddress = "0x0000000000000000000000000000000000000815";
    const localAssets = [
      "0xfffffffe88fc55121922ed6780b7e0c8ab0374d2",
      "0xfffffffed4524df96c487c197dc9a964543678a6",
      "0xfffffffeb184a34c1a2ac403f5afae19d811de72",
      "0xfffffffe221e31a529b1a5fc4101352398974a06",
      "0xfffffffe960a8ea5ccee2720828ebe2dbd97b69a",
      "0xfffffffeb1de27b22dcb3a2e1def6d0edefa3bff",
      "0xfffffffe0b00feb2a14ad90d9d4d871eba59dd9c",
      "0xfffffffe47b78475160da680caef70959e027bee",
      "0xfffffffeff0e8e9e2288fcce5e70afa61baec79d",
      "0xfffffffe4b980819e301da7d7b08b1a64299c5ff",
      "0xfffffffef0608962d5801b39f437d4cc416939e1",
      "0xfffffffefd9d0bf45a2947a519a741c4b9e99eb6",
      "0xfffffffec002b88d59aa53e9612a00fb2a2d1504",
      "0xfffffffe028682475e21a368c4f6e368efe598f0",
      "0xfffffffe6d1492e39f1674f65a6f600b4589abd7",
    ];
    const notLocalAssets = [
      "0xfffffffe88fc55121922ed6780b7e0c8ab0374d3",
      "0xfffffffe221e31a529b1a5fc4101352398974a07",
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
      test: async function () {
        const addresses = [
          ...localAssets.map((address) => ({ address, local: true })),
          ...notLocalAssets.map((address) => ({ address, local: false })),
        ];

        // Call every address and ensure it only local asset addresses get reverted
        for (const { address, local } of addresses) {
          const rawSigned = await createEthersTransaction(context, {
            to: address,
            data: encodeFunctionData({
              abi: erc20Abi,
              functionName: "approve",
              args: [ALITH_ADDRESS, 1000],
            }),
          });

          const { result } = await context.createBlock(rawSigned);
          const receipt = await context
            .viem("public")
            .getTransactionReceipt({ hash: result?.hash as `0x${string}` });

          if (local) {
            // The transaction status should be "reverted" when calling a local asset
            expect(receipt.status).to.equal("reverted");

            // Assert expected revert reason
            const revertReason = await extractRevertReason(context, result!.hash);
            expect(revertReason).to.equal("Removed precompile");
          } else {
            // The transaction status should be "success"
            expect(receipt.status).to.equal("success");
          }
        }
      },
    });

    it({
      id: "T02",
      title: "Local assets should be inactive in the precompile registry",
      test: async function () {
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
