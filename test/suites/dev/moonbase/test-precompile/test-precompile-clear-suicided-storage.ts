import {
  describeSuite,
  expect,
  deployCreateCompiledContract,
  fetchCompiledContract,
  beforeEach,
} from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import { expectEVMResult, expectOk, extractRevertReason } from "../../../../helpers";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D012929",
  title: "Precompile - Clear Suicided Storage",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let api: ApiPromise;
    let abi: Abi;
    let precompile_abi: Abi;
    let contractAddress: `0x${string}`;
    // StorageCleaner precompile address
    const precompileAddress: `0x${string}` = "0x0000000000000000000000000000000000000403";

    beforeEach(async () => {
      api = context.polkadotJs();
      const storageContract = fetchCompiledContract("Storage");
      abi = storageContract.abi;
      const storageCleanerPrecompile = fetchCompiledContract("StorageCleaner");
      precompile_abi = storageCleanerPrecompile.abi;
      const deployResult = await deployCreateCompiledContract(context, "Storage");
      contractAddress = deployResult.contractAddress;

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
    });

    it({
      id: "T01",
      title: "Should not be able to clear storage entries of a contract that is not suicided.",
      test: async function () {
        const rawTxn = await createEthersTransaction(context, {
          to: precompileAddress,
          data: encodeFunctionData({
            abi: precompile_abi,
            args: [[contractAddress], 201],
            functionName: "clearSuicidedStorage",
          }),
          gasLimit: 13_000_000,
        });
        const result = await context.createBlock(rawTxn);

        expectEVMResult(result.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(context, result.result!.hash);
        expect(revertReason).to.contain(`NotSuicided:`);
      },
    });

    it({
      id: "T02",
      title: "Should clear storage entries of a suicided contract.",
      test: async function () {
        // Add contract to the suicided list
        await context.createBlock(
          api.tx.sudo.sudo(
            api.tx.system.setStorage([[api.query.evm.suicided.key(contractAddress), null]])
          )
        );
        const suicidedContracts = await context.polkadotJs().query.evm.suicided(contractAddress);
        expect(suicidedContracts.isSome).to.be.true;
        // Call the precompile to delete the storage entries
        const rawTxn = await createEthersTransaction(context, {
          to: precompileAddress,
          data: encodeFunctionData({
            abi: precompile_abi,
            args: [[contractAddress], 201],
            functionName: "clearSuicidedStorage",
          }),
          gasLimit: 13_000_000,
        });

        await expectOk(context.createBlock(rawTxn));
        const postClearSuicidedContracts = await context
          .polkadotJs()
          .query.evm.suicided(contractAddress);
        expect(postClearSuicidedContracts.isNone).to.be.true;
      },
    });
  },
});
