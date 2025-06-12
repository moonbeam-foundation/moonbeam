import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  MIN_GLMR_DELEGATOR,
  PRECOMPILES,
  baltathar,
} from "@moonwall/util";
import { nToHex } from "@polkadot/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D022863",
  title: "Smart Contract Precompile Call - AddProxy Staking",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let testContractAddress: `0x${string}`;
    beforeEach(async function () {
      const { contractAddress } = await context.deployContract!("SmartContractPrecompileCallTest");
      testContractAddress = contractAddress;

      await context.createBlock(
        context
          .polkadotJs()
          .tx.proxy.addProxy(testContractAddress, "Staking", 0)
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
          functionName: "callAddProxy",
          gas: 10_000_000n,
          args: [BALTATHAR_ADDRESS],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);

        expectEVMResult(result!.events, "Revert");
        expect(
          async () =>
            await context.writeContract!({
              contractAddress: testContractAddress,
              contractName: "SmartContractPrecompileCallTest",
              functionName: "callAddProxy",
              args: [BALTATHAR_ADDRESS],
            })
        ).rejects.toThrowError("Function not callable by smart contracts");
      },
    });

    it({
      id: "T02",
      title: "should succeed when caller is a smart contract",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractAddress: testContractAddress,
          contractName: "SmartContractPrecompileCallTest",
          functionName: "callProxy",
          rawTxOnly: true,
          args: [
            BALTATHAR_ADDRESS,
            PRECOMPILES.ParachainStaking,
            encodeFunctionData({
              abi: fetchCompiledContract("ParachainStaking").abi,
              functionName: "delegateWithAutoCompound",
              args: [ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 100, 0, 0, 0],
            }),
          ],
        });

        const { result } = await context.createBlock(rawTxn);

        expectEVMResult(result!.events, "Succeed");
        const delegations = await context
          .polkadotJs()
          .query.parachainStaking.topDelegations(ALITH_ADDRESS);
        expect(delegations.toJSON()).to.deep.equal({
          delegations: [
            {
              owner: BALTATHAR_ADDRESS,
              amount: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
            },
          ],
          total: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
        });
      },
    });
  },
});
