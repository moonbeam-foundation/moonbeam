import "@moonbeam-network/api-augment";
import {
  TransactionTypes,
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
} from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData, type Abi } from "viem";
import { verifyLatestBlockFees } from "../../../../helpers";

// TODO: expand these tests to do multiple txn types when added to viem
describeSuite({
  id: "D010603",
  title: "Contract loop error",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let looperAddress: `0x${string}`;
    let looperAbi: Abi;

    beforeAll(async () => {
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "Looper");

      looperAddress = contractAddress;
      looperAbi = abi;
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: `"should return OutOfGas on inifinite loop ${txnType} call`,
        test: async function () {
          expect(
            async () =>
              await context.viem().call({
                account: ALITH_ADDRESS,
                to: looperAddress,
                data: encodeFunctionData({ abi: looperAbi, functionName: "infinite", args: [] }),
                gas: 12_000_000n,
              }),
            "Execution succeeded but should have failed"
          ).rejects.toThrowError("out of gas");
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) * 2 + 1}`,
        title: `should fail with OutOfGas on infinite loop ${txnType} transaction`,
        test: async function () {
          const rawSigned = await createEthersTransaction(context, {
            to: looperAddress,
            data: encodeFunctionData({ abi: looperAbi, functionName: "infinite", args: [] }),
            txnType,
          });

          const { result } = await context.createBlock(rawSigned);
          const receipt = await context
            .viem("public")
            .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
          expect(receipt.status).toBe("reverted");
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) * 3 + 1}`,
        title: `should fail with OutOfGas on infinite loop ${txnType} transaction - check fees`,
        test: async function () {
          const rawSigned = await createEthersTransaction(context, {
            to: looperAddress,
            data: encodeFunctionData({ abi: looperAbi, functionName: "infinite", args: [] }),
            txnType,
          });

          await context.createBlock(rawSigned);
          await verifyLatestBlockFees(context);
        },
      });
    }
  },
});
