import "@moonbeam-network/api-augment";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  MIN_GLMR_DELEGATOR,
  PRECOMPILES,
  baltathar,
  beforeEach,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "moonwall";
import { nToHex } from "@polkadot/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D022754",
  title: "Smart Contract Precompile Call - Proxy - Any Proxy Type",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let testContractAddress: `0x${string}`;
    beforeEach(async function () {
      const { contractAddress } = await context.deployContract!("SmartContractPrecompileCallTest");
      testContractAddress = contractAddress;

      await context.createBlock(
        context.polkadotJs().tx.proxy.addProxy(testContractAddress, "Any", 0).signAsync(baltathar)
      );
    });

    it({
      id: "T01",
      title: "should succeed when caller is a smart contract",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractAddress: testContractAddress,
          contractName: "SmartContractPrecompileCallTest",
          functionName: "callProxy",
          args: [
            BALTATHAR_ADDRESS,
            PRECOMPILES.ParachainStaking,
            encodeFunctionData({
              abi: fetchCompiledContract("ParachainStaking").abi,
              functionName: "delegateWithAutoCompound",
              args: [ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 100, 0, 0, 0],
            }),
          ],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);

        expectEVMResult(result!.events, "Succeed");
        const delegations = await context
          .polkadotJs()
          .query.parachainStaking.topDelegations(ALITH_ADDRESS);
        expect(delegations.toJSON()).to.deep.equal({
          delegations: [
            {
              owner: baltathar.address,
              amount: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
            },
          ],
          total: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
        });
      },
    });
  },
});
