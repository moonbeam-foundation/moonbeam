import {
  describeSuite,
  expect,
  deployCreateCompiledContract,
  fetchCompiledContract,
} from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectOk } from "../../../../helpers";

describeSuite({
  id: "D5001",
  title: "Lazy Migrations Pallet - Clear Suicided Storage",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title:
        "Should clear storage entries of multiple suicided contracts within the deletion limit.",
      test: async function () {
        const contracts: { address: `0x${string}`; storageKey: string }[] = [];
        const { abi } = fetchCompiledContract("Storage");

        for (let i = 0; i < 3; i++) {
          const { contractAddress } = await deployCreateCompiledContract(context, "Storage");
          const storageKey = context.polkadotJs().query.evm.accountCodes.key(contractAddress);
          contracts.push({ address: contractAddress, storageKey });

          // Create storage entries for the contract
          const rawSigned = await createEthersTransaction(context, {
            to: contractAddress,
            data: encodeFunctionData({
              abi,
              args: [0, 200],
              functionName: "store",
            }),
            gasLimit: 13_000_000,
          });
          await expectOk(context.createBlock(rawSigned));

          await context.createBlock();

          // Delete the contract to make it a suicided contract
          const rawTx = await createEthersTransaction(context, {
            to: contractAddress,
            data: encodeFunctionData({
              abi,
              functionName: "destroy",
            }),
            gasLimit: 2_000_000,
          });
          const { result } = await context.createBlock(rawTx);
          const receipt = await context
            .viem("public")
            .getTransactionReceipt({ hash: result?.hash as `0x${string}` });

          expect(receipt.status).toBe("success");

          // Call the extrinsic to delete the storage entries
          await expectOk(
            context.createBlock(
              context
                .polkadotJs()
                .tx.moonbeamLazyMigrations.clearSuicidedStorage([contractAddress], 199)
            )
          );
        }
      },
    });
  },
});
