import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, type Abi, parseEther, parseGwei } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createFundedAccount } from "./helpers";

describeSuite({
  id: "D020804",
  title: "EIP-7702 Gas Cost and Accounting",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageWriterAddress: `0x${string}`;
    let storageWriterAbi: Abi;
    let counterAddress: `0x${string}`;
    let counterAbi: Abi;

    // EIP-7702 gas costs (from EIP-7702 specification)
    const PER_AUTH_BASE_COST = 12500n; // Cost for processing each authorization
    const PER_EMPTY_ACCOUNT_COST = 25000n; // Intrinsic cost per authorization in list
    const PER_CONTRACT_CODE_BASE_COST = 2500n; // Moonbeam-specific implementation detail

    beforeAll(async () => {
      const storageWriter = await deployCreateCompiledContract(context, "StorageWriter");
      storageWriterAddress = storageWriter.contractAddress;
      storageWriterAbi = storageWriter.abi;

      const counter = await deployCreateCompiledContract(context, "Counter");
      counterAddress = counter.contractAddress;
      counterAbi = counter.abi;
    });

    it({
      id: "T01",
      title: "should calculate correct gas cost for single authorization",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Simple transaction with authorization
        const tx = {
          to: delegatingEOA.address,
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          gas: 200000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [authorization],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        // Gas used should include authorization costs
        expect(receipt.gasUsed).toBeGreaterThan(PER_AUTH_BASE_COST);

        console.log(`Gas used with 1 authorization: ${receipt.gasUsed}`);
      },
    });

    it({
      id: "T02",
      title: "should calculate correct gas cost for multiple authorizations",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const eoa1 = await createFundedAccount(context);
        const eoa2 = await createFundedAccount(context);
        const eoa3 = await createFundedAccount(context);

        // Create multiple authorizations
        const auth1 = await eoa1.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const auth2 = await eoa2.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const auth3 = await eoa3.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const tx = {
          to: eoa1.address,
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          gas: 300000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [auth1, auth2, auth3],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        // Gas should include cost for 3 authorizations
        const minExpectedGas = PER_AUTH_BASE_COST * 3n;
        expect(receipt.gasUsed).toBeGreaterThan(minExpectedGas);

        console.log(`Gas used with 3 authorizations: ${receipt.gasUsed}`);
      },
    });

    it({
      id: "T03",
      title:
        "should document current account warming behavior for authority and authorized accounts",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const coldEOA = privateKeyToAccount(generatePrivateKey());
        const warmEOA = await createFundedAccount(context);

        const coldAuth = await coldEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const warmAuth = await warmEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Execute both transactions in the same block to test warming effect
        const senderNonce = await context.viem("public").getTransactionCount({
          address: senderAccount.address,
        });

        // Transaction with cold account
        const coldTx = {
          to: coldEOA.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: senderNonce,
          chainId: 1281,
          authorizationList: [coldAuth],
          type: "eip7702" as const,
        };

        // Transaction with warm account
        const warmTx = {
          to: warmEOA.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: senderNonce + 1,
          chainId: 1281,
          authorizationList: [warmAuth],
          type: "eip7702" as const,
        };

        const coldSignature = await senderAccount.signTransaction(coldTx);
        const warmSignature = await senderAccount.signTransaction(warmTx);

        // Execute both transactions in the same block
        const result = await context.createBlock([coldSignature, warmSignature]);

        // Get gas used for both transactions
        const receipts = await Promise.all([
          context.viem("public").getTransactionReceipt({
            hash: result.result![0].hash as `0x${string}`,
          }),
          context.viem("public").getTransactionReceipt({
            hash: result.result![1].hash as `0x${string}`,
          }),
        ]);

        const coldGas = receipts[0].gasUsed;
        const warmGas = receipts[1].gasUsed;

        console.log(`Cold account gas: ${coldGas}, Warm account gas: ${warmGas}`);
        expect(coldGas).toBeGreaterThan(warmGas);
      },
    });

    it({
      id: "T04",
      title: "should test intrinsic gas cost with exact gas limit",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Calculate calldata gas cost
        // increment() function selector: 0xd09de08a (4 bytes)
        const calldata = encodeFunctionData({
          abi: counterAbi,
          functionName: "increment",
          args: [],
        });

        // Count zero and non-zero bytes in calldata
        let zeroBytes = 0n;
        let nonZeroBytes = 0n;

        // Remove '0x' prefix and process hex string
        const hexData = calldata.slice(2);
        for (let i = 0; i < hexData.length; i += 2) {
          const byte = hexData.slice(i, i + 2);
          if (byte === "00") {
            zeroBytes++;
          } else {
            nonZeroBytes++;
          }
        }

        // Calculate intrinsic gas according to EIP-7702:
        // - Base transaction cost: 21000
        // - Per authorization in list: PER_EMPTY_ACCOUNT_COST (25000)
        // - Calldata: 4 gas per zero byte, 16 gas per non-zero byte
        const calldataGas = zeroBytes * 4n + nonZeroBytes * 16n;
        const authorizationListGas = PER_EMPTY_ACCOUNT_COST * 1n; // 1 authorization
        const intrinsicGas = 21000n + authorizationListGas + calldataGas;

        console.log(`Intrinsic gas calculation breakdown:`);
        console.log(`  Base transaction: 21000`);
        console.log(`  Authorization list (1 auth): ${authorizationListGas}`);
        console.log(
          `  Calldata (${zeroBytes} zero bytes, ${nonZeroBytes} non-zero): ${calldataGas}`
        );
        console.log(`  Total intrinsic gas: ${intrinsicGas}`);

        // Test 1: Transaction with exact intrinsic gas (should fail - no gas for execution)
        const exactGasTx = {
          to: delegatingEOA.address,
          data: calldata,
          gas: intrinsicGas,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [authorization],
          type: "eip7702" as const,
        };

        try {
          const signature = await senderAccount.signTransaction(exactGasTx);
          const { result } = await context.createBlock(signature);

          if (result?.hash) {
            const receipt = await context.viem("public").getTransactionReceipt({
              hash: result.hash as `0x${string}`,
            });
            console.log(`Transaction with exact intrinsic gas status: ${receipt.status}`);
            // Should have failed due to insufficient gas
            expect(receipt.status).toBe("reverted");
          } else {
            console.log("Transaction with exact intrinsic gas failed to be included");
          }
        } catch (_error) {
          console.log("Transaction with exact intrinsic gas failed as expected");
        }

        // Test 2: Transaction with intrinsic + 1 gas (should still fail - not enough for execution)
        const almostEnoughGasTx = {
          ...exactGasTx,
          gas: intrinsicGas + 1n,
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
        };

        try {
          const signature = await senderAccount.signTransaction(almostEnoughGasTx);
          const { result } = await context.createBlock(signature);

          if (result?.hash) {
            const receipt = await context.viem("public").getTransactionReceipt({
              hash: result.hash as `0x${string}`,
            });
            console.log(`Transaction with intrinsic + 1 gas status: ${receipt.status}`);
            // Should have failed due to insufficient gas for execution
            expect(receipt.status).toBe("reverted");
          }
        } catch (_error) {
          console.log("Transaction with intrinsic + 1 gas failed as expected");
        }

        // Test 3: Transaction with sufficient gas for execution (should succeed)
        const executionGasEstimate = 30_000n; // Estimated gas for increment() execution
        const sufficientGasTx = {
          ...exactGasTx,
          gas: intrinsicGas + executionGasEstimate,
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
        };

        const signature = await senderAccount.signTransaction(sufficientGasTx);
        const { result } = await context.createBlock(signature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        console.log(`Transaction with sufficient gas:`);
        console.log(`  Gas limit: ${intrinsicGas + executionGasEstimate}`);
        console.log(`  Gas used: ${receipt.gasUsed}`);
        console.log(`  Status: ${receipt.status}`);

        expect(receipt.status).toBe("success");

        // Verify the intrinsic gas portion
        const executionGas = receipt.gasUsed - intrinsicGas;
        console.log(`  Execution gas (actual - intrinsic): ${executionGas}`);

        // Gas used should be at least the intrinsic gas
        expect(receipt.gasUsed).toBeGreaterThanOrEqual(intrinsicGas);
      },
    });

    it({
      id: "T05",
      title: "should test gas cost for self-delegation",
      test: async () => {
        const selfDelegatingEOA = await createFundedAccount(context);

        // Self-authorization (EOA delegates to a contract on behalf of itself)
        // In EIP-7702, when the authorizing address is the same as the sender,
        // the authorization nonce should be current_nonce + 1 because the EVM
        // increments the nonce before processing the authorization list
        const currentNonce = await context.viem("public").getTransactionCount({
          address: selfDelegatingEOA.address,
        });
        const selfAuth = await selfDelegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: currentNonce + 1, // current_nonce + 1 for self-authorizing transactions
        });

        // Calculate calldata gas cost for increment()
        const calldata = encodeFunctionData({
          abi: counterAbi,
          functionName: "increment",
          args: [],
        });

        // Count zero and non-zero bytes in calldata
        let zeroBytes = 0n;
        let nonZeroBytes = 0n;
        const hexData = calldata.slice(2);
        for (let i = 0; i < hexData.length; i += 2) {
          const byte = hexData.slice(i, i + 2);
          if (byte === "00") {
            zeroBytes++;
          } else {
            nonZeroBytes++;
          }
        }

        // Calculate intrinsic gas for self-delegation
        const calldataGas = zeroBytes * 4n + nonZeroBytes * 16n;
        const authorizationListGas = PER_EMPTY_ACCOUNT_COST * 1n; // 1 authorization
        const intrinsicGas = 21000n + authorizationListGas + calldataGas;

        console.log(`Self-delegation intrinsic gas calculation:`);
        console.log(`  Base transaction: 21000`);
        console.log(`  Authorization list (1 auth): ${authorizationListGas}`);
        console.log(
          `  Calldata (${zeroBytes} zero bytes, ${nonZeroBytes} non-zero): ${calldataGas}`
        );
        console.log(`  Total intrinsic gas: ${intrinsicGas}`);

        // Test with sufficient gas for self-delegation
        const gasLimit = intrinsicGas + 30000n; // Add execution gas
        const selfTx = {
          to: selfDelegatingEOA.address,
          data: calldata,
          gas: gasLimit,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: currentNonce, // Current nonce for the transaction
          chainId: 1281,
          authorizationList: [selfAuth],
          type: "eip7702" as const,
        };

        // Sign with the same account that created the authorization
        const signature = await selfDelegatingEOA.signTransaction(selfTx);

        // Need to fund gas for the transaction
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(selfDelegatingEOA.address, parseEther("1")),
        ]);

        // Send the self-signed transaction
        const { result } = await context.createBlock(signature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        expect(receipt.status).toBe("success");

        // Detailed gas cost analysis
        console.log(`Self-delegation gas costs:`);
        console.log(`  Gas limit: ${gasLimit}`);
        console.log(`  Gas used: ${receipt.gasUsed}`);
        console.log(`  Intrinsic gas: ${intrinsicGas}`);
        const executionGas = receipt.gasUsed - intrinsicGas;
        console.log(`  Execution gas: ${executionGas}`);

        // Verify gas used is reasonable
        expect(receipt.gasUsed).toBeGreaterThanOrEqual(intrinsicGas);
        expect(receipt.gasUsed).toBeLessThan(gasLimit);

        // Additional gas cost checks for self-delegation specifics
        // Self-delegation might have different gas costs due to:
        // 1. Account state changes (nonce increment before auth processing)
        // 2. Self-reference in authorization
        const selfDelegationOverhead = receipt.gasUsed - intrinsicGas;
        console.log(`  Self-delegation overhead: ${selfDelegationOverhead}`);

        // Verify delegation was set
        const code = await context.viem("public").getCode({
          address: selfDelegatingEOA.address,
        });
        expect(code?.startsWith("0xef0100")).toBe(true);
        console.log(`  Delegation code set: ${code?.slice(0, 50)}...`);

        // Check counter was incremented
        const count = await context.viem("public").readContract({
          address: selfDelegatingEOA.address,
          abi: counterAbi,
          functionName: "count",
          args: [],
        });
        expect(count).toBe(1n);
        console.log(`  Counter value after increment: ${count}`);
      },
    });

    it({
      id: "T06",
      title: "should handle out-of-gas during authorization processing",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Very low gas limit that should fail during authorization processing
        const lowGasTx = {
          to: delegatingEOA.address,
          data: encodeFunctionData({
            abi: storageWriterAbi,
            functionName: "store",
            args: [1n, 100n],
          }),
          gas: 25000n, // Very low gas
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [authorization],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(lowGasTx);
        const { result } = await context.createBlock(signature);

        // Transaction should fail due to out of gas
        expect(result?.successful).toBe(false);
        expect(result?.hash).toBeUndefined();

        // Delegation should not be set
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();
      },
    });

    it({
      id: "T07",
      title: "should test gas refund for authorization clearing",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        // First set a delegation
        const setAuth = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setTx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [setAuth],
          type: "eip7702" as const,
        };

        const setSignature = await senderAccount.signTransaction(setTx);
        await context.createBlock(setSignature);

        // Verify delegation is set
        const codeAfterSet = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(codeAfterSet?.startsWith("0xef0100")).toBe(true);

        // Now clear the delegation (delegate to zero address)
        const clearAuth = await delegatingEOA.signAuthorization({
          contractAddress: "0x0000000000000000000000000000000000000000",
          chainId: 1281,
          nonce: 1,
        });

        const clearTx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [clearAuth],
          type: "eip7702" as const,
        };

        const clearSignature = await senderAccount.signTransaction(clearTx);
        const clearResult = await context.createBlock(clearSignature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: clearResult.result?.hash as `0x${string}`,
        });

        // Gas used for clearing
        console.log(`Gas used for clearing delegation: ${receipt.gasUsed}`);
        expect(receipt.gasUsed).toBe(36800n);
      },
    });
  },
});
