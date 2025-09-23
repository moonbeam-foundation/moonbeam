import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  CONTRACT_PROXY_TYPE_ANY,
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  PRECOMPILE_PROXY_ADDRESS,
} from "@moonwall/util";
import { expectEVMResult } from "helpers/eth-transactions";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D023504",
  title: "Storage growth limit - Precompiles",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should fail to addProxy due to insufficient gas required to cover the storage growth",
      test: async () => {
        const { abi: proxyAbi } = fetchCompiledContract("Proxy");

        const estimatedGas = await context.viem().estimateGas({
          account: FAITH_ADDRESS,
          to: PRECOMPILE_PROXY_ADDRESS,
          data: encodeFunctionData({
            abi: proxyAbi,
            functionName: "addProxy",
            args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          }),
        });

        // Snapshot estimated gas
        expect(estimatedGas).toMatchInlineSnapshot(`49920n`);

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          privateKey: FAITH_PRIVATE_KEY,
          rawTxOnly: true,
          gas: estimatedGas - 5_000n,
        });
        const { result } = await context.createBlock(rawTxn);
        // Check that the transaction failed with an out of gas error
        expectEVMResult(result!.events, "Error", "OutOfGas");
      },
    });

    it({
      id: "T02",
      title: "should addProxy correctly with the required gas to cover the storage growth",
      test: async () => {
        const { abi: proxyAbi } = fetchCompiledContract("Proxy");

        const estimatedGas = await context.viem().estimateGas({
          account: FAITH_ADDRESS,
          to: PRECOMPILE_PROXY_ADDRESS,
          data: encodeFunctionData({
            abi: proxyAbi,
            functionName: "addProxy",
            args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          }),
        });

        // Snapshot estimated gas
        expect(estimatedGas).toMatchInlineSnapshot(`50666n`);

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          privateKey: FAITH_PRIVATE_KEY,
          rawTxOnly: true,
          gas: estimatedGas,
        });

        const { result } = await context.createBlock(rawTxn);
        // Check that the transaction failed with an out of gas error
        expectEVMResult(result!.events, "Succeed");

        const { gasUsed } = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        // The tx can create an account, so record 148 bytes of storage growth
        // Storage growth ratio is 366
        expect(gasUsed).toMatchInlineSnapshot(`31032n`);
      },
    });
  },
});
