import "@moonbeam-network/api-augment";
import {
  TransactionTypes,
  deployCreateCompiledContract,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { hexToU8a } from "@polkadot/util";
import { encodeDeployData, keccak256, numberToHex, toRlp } from "viem";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

// TODO: expand these tests to do multiple txn types when added to viem
describeSuite({
  id: "D0601",
  title: "Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: `should return the ${txnType} transaction hash`,
        test: async function () {
          const { hash } = await deployCreateCompiledContract(context, "MultiplyBy7");
          await context.createBlock();
          expect(hash).toBeTruthy();
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 4}`,
        title: `${txnType} should return the contract code`,
        test: async () => {
          const contractData = fetchCompiledContract("MultiplyBy7");
          const callCode = (await context.viem().call({ data: contractData.bytecode })).data;
          const { contractAddress } = await deployCreateCompiledContract(context, "MultiplyBy7");
          const deployedCode = await context
            .viem("public")
            .getBytecode({ address: contractAddress! });
          expect(callCode).to.be.eq(deployedCode);
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 7}`,
        title: `should not contain ${txnType}  contract at genesis`,
        test: async function () {
          const { contractAddress } = await deployCreateCompiledContract(context, "MultiplyBy7");
          expect(
            await context.viem().getBytecode({ address: contractAddress!, blockNumber: 0n })
          ).toBeUndefined();
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 10}`,
        title: `${txnType} deployed contracts should store the code on chain`,
        test: async function () {
          // This is to enable pending tag support
          await context.createBlock();
          const compiled = fetchCompiledContract("MultiplyBy7");
          const callData = encodeDeployData({
            abi: compiled.abi,
            bytecode: compiled.bytecode,
            args: [],
          }) as `0x${string}`;

          const nonce = await context
            .viem("public")
            .getTransactionCount({ address: ALITH_ADDRESS });

          await context.viem().sendTransaction({ data: callData, nonce });

          const contractAddress = ("0x" +
            keccak256(hexToU8a(toRlp([ALITH_ADDRESS, numberToHex(nonce)])))
              .slice(12)
              .substring(14)) as `0x${string}`;

          expect(
            await context
              .viem("public")
              .getBytecode({ address: contractAddress, blockTag: "pending" })
          ).to.deep.equal(compiled.deployedBytecode);

          await context.createBlock();

          expect(
            await context
              .viem("public")
              .getBytecode({ address: contractAddress, blockTag: "latest" })
          ).to.deep.equal(compiled.deployedBytecode);
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 13}`,
        title: `should check latest block fees for ${txnType}`,
        test: async function () {
          await context.createBlock();
          await deployCreateCompiledContract(context, "Fibonacci", { maxPriorityFeePerGas: 0n });
          await verifyLatestBlockFees(context);
        },
      });
    }
  },
});
