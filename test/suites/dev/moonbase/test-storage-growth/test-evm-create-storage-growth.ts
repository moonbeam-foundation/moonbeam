import "@moonbeam-network/api-augment";
import { TransactionTypes, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { expectEVMResult } from "helpers/eth-transactions";

describeSuite({
  id: "D013502",
  title: "Storage growth limit - Contract Creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    // This is the gas cost of the transaction that deploys the Fibonacci contract:
    // (Account Code Size (112) + Length of the bytecode (550)) * Storage Growth Gas Ratio (366)
    // The length of the bytecode is in the generate Fibonacci.json
    // file at 'contract.evm.deployedBytecode.object'
    const EXPECTED_STORAGE_GROWTH_GAS = 242_292;
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: "should out of gas when gas provided is not enough to cover storage growth",
        test: async function () {
          const { bytecode } = fetchCompiledContract("Fibonacci");
          // Deploy contract with insufficient gas limit
          const rawSigned = await createEthersTransaction(context, {
            account: ALITH_ADDRESS,
            data: bytecode,
            gasLimit: EXPECTED_STORAGE_GROWTH_GAS - 1,
          });

          const { result } = await context.createBlock(rawSigned);
          // Check that the transaction failed with an out of gas error
          expectEVMResult(result!.events, "Error", "OutOfGas");
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 4}`,
        title: "should estimate enough gas to cover storage growth",
        test: async function () {
          const estimatedGas = await context.viem().estimateGas({
            account: ALITH_ADDRESS,
            data: fetchCompiledContract("Fibonacci").bytecode,
          });

          expect(estimatedGas).toBeGreaterThanOrEqual(EXPECTED_STORAGE_GROWTH_GAS);
          expect(estimatedGas).toBe(263644n);
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 7}`,
        title: "should deploy contract with enough gas to cover storage growth",
        test: async function () {
          const contractData = fetchCompiledContract("Fibonacci");
          const callCode = (
            await context.viem().call({ data: contractData.bytecode, gas: 245_586n })
          ).data;
          const { contractAddress } = await context.deployContract!("Fibonacci");
          const deployedCode = await context
            .viem("public")
            .getBytecode({ address: contractAddress! });
          expect(callCode).to.be.eq(deployedCode);
        },
      });
    }
  },
});
