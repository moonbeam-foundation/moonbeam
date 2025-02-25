import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CONTRACT_PROXY_TYPE_ANY,
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  PRECOMPILE_NATIVE_ERC20_ADDRESS,
  PRECOMPILE_PROXY_ADDRESS,
} from "@moonwall/util";
import { parseEther } from "ethers";
import { expectEVMResult } from "helpers/eth-transactions";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D013504",
  title: "Storage growth limit - Precompiles",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const newAccount = "0x1ced798a66b803d0dbb665680283980a939a6432";

    it({
      id: "T01",
      title: "should fail transfer due to insufficient gas required to cover the storage growth",
      test: async () => {
        const { abi: ierc20Abi } = fetchCompiledContract("IERC20");
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
        expect(estimatedGas).toMatchInlineSnapshot(`103826n`);

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          privateKey: FAITH_PRIVATE_KEY,
          rawTxOnly: true,
          gas: estimatedGas,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const proxyProxyEstimatedGas = await context.viem().estimateGas({
          account: BALTATHAR_ADDRESS,
          to: PRECOMPILE_PROXY_ADDRESS,
          data: encodeFunctionData({
            abi: proxyAbi,
            functionName: "proxy",
            args: [
              FAITH_ADDRESS,
              PRECOMPILE_NATIVE_ERC20_ADDRESS,
              encodeFunctionData({
                abi: ierc20Abi,
                functionName: "transfer",
                args: [newAccount, parseEther("5")],
              }),
            ],
          }),
        });

        // Snapshot estimated gas
        expect(proxyProxyEstimatedGas).toMatchInlineSnapshot(`92232n`);

        const balBefore = await context.viem().getBalance({ address: FAITH_ADDRESS });
        const rawTxn2 = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "proxy",
          args: [
            FAITH_ADDRESS,
            PRECOMPILE_NATIVE_ERC20_ADDRESS,
            encodeFunctionData({
              abi: ierc20Abi,
              functionName: "transfer",
              args: [newAccount, parseEther("5")],
            }),
          ],
          privateKey: BALTATHAR_PRIVATE_KEY,
          rawTxOnly: true,
          gas: proxyProxyEstimatedGas - 10_000n,
        });

        const { result: result2 } = await context.createBlock(rawTxn2);
        // Check that the transaction failed with an out of gas error
        expectEVMResult(result2!.events, "Error", "OutOfGas");

        const balAfter = await context.viem().getBalance({ address: FAITH_ADDRESS });
        expect(balBefore - balAfter).to.equal(0n);
      },
    });

    it({
      id: "T02",
      title: "should transfer correctly with the required gas to cover the storage growth",
      test: async () => {
        const balBefore = await context.viem().getBalance({ address: FAITH_ADDRESS });
        const { abi: ierc20Abi } = fetchCompiledContract("IERC20");
        const { abi: proxyAbi } = fetchCompiledContract("Proxy");

        const estimatedGas = await context.viem().estimateGas({
          account: BALTATHAR_ADDRESS,
          to: PRECOMPILE_PROXY_ADDRESS,
          data: encodeFunctionData({
            abi: proxyAbi,
            functionName: "proxy",
            args: [
              FAITH_ADDRESS,
              PRECOMPILE_NATIVE_ERC20_ADDRESS,
              encodeFunctionData({
                abi: ierc20Abi,
                functionName: "transfer",
                args: [newAccount, parseEther("5")],
              }),
            ],
          }),
        });

        // Snapshot estimated gas
        expect(estimatedGas).toMatchInlineSnapshot(`92232n`);

        const rawTxn2 = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "proxy",
          args: [
            FAITH_ADDRESS,
            PRECOMPILE_NATIVE_ERC20_ADDRESS,
            encodeFunctionData({
              abi: ierc20Abi,
              functionName: "transfer",
              args: [newAccount, parseEther("5")],
            }),
          ],
          privateKey: BALTATHAR_PRIVATE_KEY,
          rawTxOnly: true,
          gas: estimatedGas,
        });

        const { result } = await context.createBlock(rawTxn2);
        // Check that the transaction failed with an out of gas error
        expectEVMResult(result!.events, "Succeed");

        const { gasUsed } = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        // The tx can create an account, so record 148 bytes of storage growth
        // Storage growth ratio is 366
        // storage_gas = 148 * 366 = 54168
        // pov_gas = proof_size * GAS_LIMIT_POV_RATIO
        expect(gasUsed).toMatchInlineSnapshot(`59392n`);

        const balAfter = await context.viem().getBalance({ address: FAITH_ADDRESS });
        expect(balBefore - balAfter).to.equal(parseEther("5"));
      },
    });
  },
});
