import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  MIN_GLMR_DELEGATOR,
  PRECOMPILES,
  baltathar,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D012979",
  title: "Smart Contract Precompile Call - Proxy - Any Proxy Type",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let testContractAddress: `0x${string}`;
    beforeEach(async function () {
      const { contractAddress } = await context.deployContract!("SmartContractPrecompileCallTest");
      testContractAddress = contractAddress;

      await context.createBlock(
        context
          .polkadotJs()
          .tx.proxy.addProxy(testContractAddress, "Governance", 0)
          .signAsync(baltathar)
      );
    });

    it({
      id: "T01",
      title: "should revert when caller is a smart contract",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractAddress: testContractAddress,
          contractName: "SmartContractPrecompileCallTest",
          functionName: "callProxy",
          gas: 5_000_000n,
          rawTxOnly: true,
          args: [
            BALTATHAR_ADDRESS,
            PRECOMPILES["ParachainStaking"],
            encodeFunctionData({
              abi: fetchCompiledContract("ParachainStaking").abi,
              functionName: "delegateWithAutoCompound",
              args: [ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 100, 0, 0, 0],
            }),
          ],
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");

        expect(
          async () =>
            await context.writeContract!({
              contractAddress: testContractAddress,
              contractName: "SmartContractPrecompileCallTest",
              functionName: "callProxy",
              args: [
                BALTATHAR_ADDRESS,
                PRECOMPILES["ParachainStaking"],
                encodeFunctionData({
                  abi: fetchCompiledContract("ParachainStaking").abi,
                  functionName: "delegateWithAutoCompound",
                  args: [ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 100, 0, 0, 0],
                }),
              ],
            })
        ).rejects.toThrowError("CallFiltered");
      },
    });
  },
});
