import {
  describeSuite,
  expect,
  deployCreateCompiledContract,
  fetchCompiledContract,
  beforeAll,
} from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectOk } from "../../../../helpers";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D012201",
  title: "Lazy Migrations Pallet - Clear Suicided Storage",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();
    });

    it({
      id: "T01",
      title:
        "Should clear storage entries of multiple suicided contracts within the deletion limit.",
      test: async function () {
        const { abi } = fetchCompiledContract("Storage");

        for (let i = 0; i < 3; i++) {
          const { contractAddress } = await deployCreateCompiledContract(context, "Storage");

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

          // Call the extrinsic to delete the storage entries
          const tx = await context.createBlock(
            api.tx.moonbeamLazyMigrations.clearSuicidedStorage([contractAddress], 199)
          );
          await expect(!tx.result?.successful, "The contract storage cannot be removed");

          // Remove The contract code to make it corrupted
          await context.createBlock(
            api.tx.sudo.sudo(
              api.tx.system.killStorage([api.query.evm.accountCodes.key(contractAddress)])
            )
          );

          // Now, the storage can be removed
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
