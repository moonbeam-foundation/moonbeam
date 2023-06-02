import "@moonbeam-network/api-augment";
import {
  EthTransactionType,
  TransactionTypes,
  beforeAll,
  describeSuite,
  expect,
} from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTxn, faith, getCompiled } from "@moonwall/util";
import { AbiConstructor } from "abitype";
import { randomBytes } from "ethers";
import { encodeDeployData } from "viem";
import { customDevRpcRequest } from "../../../helpers/common.js";
import { getAllContracts } from "../../../helpers/contracts.js";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";

describeSuite({
  id: "D1702",
  title: "Estimate Gas - Multiply",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const contractNames = getAllContracts();

    beforeAll(async function () {
      // Estimation for storage need to happen in a block > than genesis.
      // Otherwise contracts that uses block number as storage will remove instead of storing
      // (as block.number == H256::default).
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should have at least 1 contract to estimate",
      test: async function () {
        expect(contractNames).length.to.be.at.least(1);
      },
    });

    const calculateTestCaseNumber = (contractName: string, txnType: EthTransactionType) =>
      contractNames.indexOf(contractName) * TransactionTypes.length +
      TransactionTypes.indexOf(txnType) +
      2;

    for (const contractName of contractNames) {
      for (const txnType of TransactionTypes) {
        it({
          id: `T${calculateTestCaseNumber(contractName, txnType).toString().padStart(2, "0")}`,
          title: `should be enough for contract ${contractName} via ${txnType}`,
          test: async function () {
            const { byteCode, contract } = getCompiled(contractName);
            const constructorAbi = contract.abi.find(
              (call) => call.type == "constructor"
            ) as AbiConstructor;
            // ask RPC for an gas estimate of deploying this contract

            const args = constructorAbi
              ? constructorAbi.inputs.map((input) =>
                  input.type == "bool"
                    ? true
                    : input.type == "address"
                    ? faith.address
                    : input.type.startsWith("uint")
                    ? `0x${Buffer.from(
                        randomBytes(Number(input.type.split("uint")[1]) / 8)
                      ).toString("hex")}`
                    : input.type.startsWith("bytes")
                    ? `0x${Buffer.from(randomBytes(Number(input.type.split("bytes")[1]))).toString(
                        "hex"
                      )}`
                    : "0x"
                )
              : [];

            const callData = encodeDeployData({
              abi: contract.abi,
              args,
              bytecode: byteCode,
            });

            let estimate: bigint;
            let creationResult: "Revert" | "Succeed";
            try {
              estimate = await customDevRpcRequest("eth_estimateGas", [
                {
                  from: ALITH_ADDRESS,
                  data: callData,
                },
              ]);
              creationResult = "Succeed";
            } catch (e: any) {
              if (e.message == "VM Exception while processing transaction: revert") {
                estimate = 12_000_000n;
                creationResult = "Revert";
              } else {
                throw e;
              }
            }

            // attempt a transaction with our estimated gas
            const { rawSigned } = await createEthersTxn(context, {
              data: callData,
              gasLimit: estimate,
              txnType,
            });
            const { result } = await context.createBlock(rawSigned);
            const receipt = await context
              .viemClient("public")
              .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

            expectEVMResult(result!.events, creationResult);
            expect(receipt.status == "success").to.equal(creationResult == "Succeed");
          },
        });
      }
    }
  },
});
