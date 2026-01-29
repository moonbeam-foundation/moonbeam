import "@moonbeam-network/api-augment";
import {
  TransactionTypes,
  describeSuite,
  expect,
  deployCreateCompiledContract,
  beforeAll,
} from "moonwall";
import { ConstantStore, EIP_7825_MAX_TRANSACTION_GAS_LIMIT } from "../../../../helpers";

describeSuite({
  id: "D010103",
  title: "Block creation - suite 2",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let specVersion: number;
    beforeAll(async () => {
      specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: `${txnType} should be allowed up to EIP-7825 transaction gas limit cap`,
        test: async function () {
          const { hash, status } = await deployCreateCompiledContract(context, "MultiplyBy7", {
            type: txnType,
            gas: EIP_7825_MAX_TRANSACTION_GAS_LIMIT,
          });
          expect(status).toBe("success");
          const receipt = await context.viem().getTransactionReceipt({ hash });
          expect(receipt.blockHash).toBeTruthy();
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) * 2 + 1}`,
        title: `${txnType} should fail exceeding EIP-7825 transaction gas limit cap`,
        test: async function () {
          await expect(
            async () =>
              await deployCreateCompiledContract(context, "MultiplyBy7", {
                type: txnType,
                gas: EIP_7825_MAX_TRANSACTION_GAS_LIMIT + 1n,
              }),
            "Transaction should be reverted but instead contract deployed"
          ).rejects.toThrowError("exceeds transaction gas limit cap");
        },
      });
    }

    it({
      id: "T07",
      title: "should be accessible within a contract",
      test: async function () {
        const { contractAddress, abi } = await context.deployContract!("BlockVariables", {
          gas: 500_000n,
        });
        expect(
          await context.viem().readContract({
            address: contractAddress!,
            abi,
            args: [],
            functionName: "getGasLimit",
          })
        ).to.equal(ConstantStore(context).GAS_LIMIT.get(specVersion));
      },
    });
  },
});
