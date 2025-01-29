import "@moonbeam-network/api-augment";
import {
  TransactionTypes,
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
} from "@moonwall/cli";
import {
  CHARLETH_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  createEthersTransaction,
  ALITH_ADDRESS,
} from "@moonwall/util";
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
          await expect(
            async () =>
              await context.viem().call({
                account: CHARLETH_ADDRESS,
                to: looperAddress,
                data: encodeFunctionData({ abi: looperAbi, functionName: "infinite", args: [] }),
                gas: 12_000_000n,
              }),
            "Execution succeeded but should have failed"
          ).rejects.toThrowError("out of gas");
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1 + TransactionTypes.length}`,
        title: `should fail with OutOfGas on infinite loop ${txnType} transaction`,
        test: async function () {
          const nonce = await context.viem().getTransactionCount({ address: CHARLETH_ADDRESS });

          const rawSigned = await createEthersTransaction(context, {
            to: looperAddress,
            data: encodeFunctionData({ abi: looperAbi, functionName: "infinite", args: [] }),
            txnType,
            nonce,
            privateKey: CHARLETH_PRIVATE_KEY,
          });

          const { result } = await context.createBlock(rawSigned, {
            signer: { type: "ethereum", privateKey: CHARLETH_PRIVATE_KEY },
          });

          expect(result.successful).to.be.true;

          const receipt = await context
            .viem("public")
            .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
          expect(receipt.status).toBe("reverted");
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1 + TransactionTypes.length * 2}`,
        title: `should fail with OutOfGas on infinite loop ${txnType} transaction - check fees`,
        test: async function () {
          const nonce = await context.viem().getTransactionCount({ address: CHARLETH_ADDRESS });

          const rawSigned = await createEthersTransaction(context, {
            to: looperAddress,
            data: encodeFunctionData({ abi: looperAbi, functionName: "infinite", args: [] }),
            txnType,
            nonce,
            privateKey: CHARLETH_PRIVATE_KEY,
          });

          const { result } = await context.createBlock(rawSigned, {
            signer: { type: "ethereum", privateKey: CHARLETH_PRIVATE_KEY },
          });

          expect(result.successful).to.be.true;
          await verifyLatestBlockFees(context);
        },
      });
    }
  },
});
