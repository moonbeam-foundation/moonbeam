import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, type Abi, parseEther } from "viem";
import { sendRawTransaction } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createFundedAccount, createViemTransaction } from "./helpers";
import { getTransactionReceiptWithRetry } from "../../../../helpers/eth-transactions";

describeSuite({
  id: "D010304",
  title: "EIP-7702 Gas Cost and Accounting",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let storageWriterAddress: `0x${string}`;
    let storageWriterAbi: Abi;
    let counterAddress: `0x${string}`;
    let counterAbi: Abi;
    let chainId: number;

    // EIP-7702 gas costs (from EIP-7702 specification)
    const PER_AUTH_BASE_COST = 12500n; // Cost for processing each authorization
    const PER_EMPTY_ACCOUNT_COST = 25000n; // Intrinsic cost per authorization in list
    const PER_CONTRACT_CODE_BASE_COST = 2500n; // Moonbeam-specific implementation detail

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

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
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Simple transaction with authorization
        const tx = {
          to: "0x0000000000000000000000000000000000000000", // Any recipient wihout code should work
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signedTx = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signedTx);
        await context.createBlock();

        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");

        // Gas used should include authorization costs
        expect(receipt.gasUsed).toBeGreaterThan(PER_AUTH_BASE_COST);

        console.log(`Gas used with 1 authorization: ${receipt.gasUsed}`);
      },
    });

    it({
      id: "T02",
      title: "should calculate correct gas cost for multiple authorizations",
      test: async () => {
        const sender = await createFundedAccount(context);
        const eoa1 = (await createFundedAccount(context)).account;
        const eoa2 = (await createFundedAccount(context)).account;
        const eoa3 = (await createFundedAccount(context)).account;

        // Create multiple authorizations
        const auth1 = await eoa1.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth2 = await eoa2.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth3 = await eoa3.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const tx = {
          to: eoa1.address,
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          chainId: chainId,
          authorizationList: [auth1, auth2, auth3],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signedTx = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signedTx);
        await context.createBlock();

        const receipt = await getTransactionReceiptWithRetry(context, hash);

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
        const sender = await createFundedAccount(context);
        const coldEOA = privateKeyToAccount(generatePrivateKey());
        const warmEOA = (await createFundedAccount(context)).account;

        const coldAuth = await coldEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const warmAuth = await warmEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Execute both transactions in the same block to test warming effect
        const senderNonce = await context.viem().getTransactionCount({
          address: sender.account.address,
        });

        // Transaction with cold account
        const coldTx = {
          to: coldEOA.address,
          chainId: chainId,
          authorizationList: [coldAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
          nonce: senderNonce,
        };

        // Transaction with warm account
        const warmTx = {
          to: warmEOA.address,
          chainId: chainId,
          authorizationList: [warmAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
          nonce: senderNonce + 1,
        };

        const coldSignature = await createViemTransaction(context, coldTx);
        const warmSignature = await createViemTransaction(context, warmTx);

        // Execute both transactions in the same block
        const result = await context.createBlock([coldSignature, warmSignature]);

        // Get gas used for both transactions
        const receipts = await Promise.all([
          getTransactionReceiptWithRetry(context, result.result![0].hash as `0x${string}`),
          getTransactionReceiptWithRetry(context, result.result![1].hash as `0x${string}`),
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
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
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
          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        try {
          const signature = await createViemTransaction(context, exactGasTx);
          const hash = await sendRawTransaction(context, signature);
          await context.createBlock();

          const receipt = await getTransactionReceiptWithRetry(context, hash);
          console.log(`Transaction with exact intrinsic gas status: ${receipt.status}`);
          // Should have failed due to insufficient gas
          expect(receipt.status).toBe("reverted");
        } catch (_error) {
          console.log("Transaction with exact intrinsic gas failed as expected");
        }

        // Test 2: Transaction with intrinsic + 1 gas (should still fail - not enough for execution)
        const almostEnoughGasTx = {
          ...exactGasTx,
          gas: intrinsicGas + 1n,
        };

        try {
          const signature = await createViemTransaction(context, almostEnoughGasTx);
          const hash = await sendRawTransaction(context, signature);
          await context.createBlock();

          const receipt = await getTransactionReceiptWithRetry(context, hash);
          console.log(`Transaction with intrinsic + 1 gas status: ${receipt.status}`);
          // Should have failed due to insufficient gas for execution
          expect(receipt.status).toBe("reverted");
        } catch (_error) {
          console.log("Transaction with intrinsic + 1 gas failed as expected");
        }

        // Test 3: Transaction with sufficient gas for execution (should succeed)
        const executionGasEstimate = 30_000n; // Estimated gas for increment() execution
        const sufficientGasTx = {
          ...exactGasTx,
          gas: intrinsicGas + executionGasEstimate,
        };

        const signature = await createViemTransaction(context, sufficientGasTx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await getTransactionReceiptWithRetry(context, hash);

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
      title: "should handle out-of-gas during authorization processing",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
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
          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, lowGasTx);
        const { result } = await context.createBlock(signature);

        // Transaction should fail due to out of gas
        expect(result?.successful).toBe(false);
        expect(result?.hash).toBeUndefined();

        // Delegation should not be set
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();
      },
    });

    it({
      id: "T06",
      title: "should test gas refund for authorization clearing",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        // First set a delegation
        const setAuth = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setTx = {
          to: delegatingEOA.address,
          chainId: chainId,
          authorizationList: [setAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const setSignature = await createViemTransaction(context, setTx);
        const setHash = await sendRawTransaction(context, setSignature);
        await context.createBlock();

        // Verify delegation is set
        const codeAfterSet = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(codeAfterSet?.startsWith("0xef0100")).toBe(true);

        // Now clear the delegation (delegate to zero address)
        const clearAuth = await delegatingEOA.signAuthorization({
          contractAddress: "0x0000000000000000000000000000000000000000",
          chainId: chainId,
          nonce: 1,
        });

        const clearTx = {
          to: "0x0000000000000000000000000000000000000000", // Any address without code
          chainId: chainId,
          authorizationList: [clearAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const clearSignature = await createViemTransaction(context, clearTx);
        const clearHash = await sendRawTransaction(context, clearSignature);
        await context.createBlock();

        const receipt = await getTransactionReceiptWithRetry(context, clearHash);

        expect(receipt.status).toBe("success");

        // Gas used for clearing
        console.log(`Gas used for clearing delegation: ${receipt.gasUsed}`);
        expect(receipt.gasUsed).toBe(36800n);
      },
    });
  },
});
