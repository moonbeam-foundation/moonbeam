import "@moonbeam-network/api-augment";
import {
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
  sendRawTransaction,
} from "moonwall";
import { keccak256, concat, encodeFunctionData, numberToHex, type Abi } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createViemTransaction } from "./helpers";
import { getTransactionReceiptWithRetry } from "../../../../helpers/eth-transactions";

describeSuite({
  id: "D010305",
  title: "EIP-7702 Transactions",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let contractAddress: `0x${string}`;
    let contractAbi: Abi;
    let chainId: number;

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

      // Deploy the delegation contract
      const { contractAddress: address, abi } = await deployCreateCompiledContract(
        context,
        "BalanceTracker"
      );

      expect(address).toBeTruthy();
      console.log(`Delegation contract deployed at: ${address}`);

      contractAddress = address;
      contractAbi = abi;
    });

    it({
      id: "T01",
      title: "happy path - should successfully delegate with valid EIP-7702 authorization",
      test: async () => {
        // Create a new EOA for delegation
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);
        const delegatingAddress = delegatingEOA.address;
        console.log(`Created EOA for delegation: ${delegatingAddress}`);

        // Fund the delegating EOA with some balance from ALITH
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingAddress, 1000000000000000000n),
        ]);

        // Set up initial delegation
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
          nonce: 0,
        });

        console.log(
          `Authorization created for ${delegatingAddress} to delegate to ${contractAddress}`
        );
        console.log(`Authorization nonce: ${authorization.nonce}`);
        console.log(`Authorization details:`, {
          contractAddress: contractAddress,
          chainId: authorization.chainId,
          nonce: authorization.nonce?.toString(),
          r: authorization.r,
          s: authorization.s,
          yParity: authorization.yParity,
        });

        // Create the authorization list
        const authorizationList = [authorization];

        // Use the delegation ABI from helpers

        // Set balance for an arbitrary address
        const targetAddress = "0x1234567890123456789012345678901234567890" as `0x${string}`;
        const targetBalance = 5000n;

        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: [targetAddress, targetBalance],
        });

        // Create a raw EIP-7702 transaction manually
        console.log(`Creating EIP-7702 transaction with authorizationList...`);
        console.log(`Authorization list being sent:`, authorizationList);

        // Create the transaction object with authorizationList
        const transaction = {
          to: delegatingAddress,
          data: callData,
          chainId: chainId,
          authorizationList,
          txnType: "eip7702" as const,
        };

        console.log(`Transaction object:`, transaction);

        // Sign the transaction
        const signature = await createViemTransaction(context, transaction);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        console.log(`Transaction submitted by ALITH for delegation to ${delegatingAddress}`);
        console.log(`Transaction hash: ${hash}`);

        // Check transaction receipt
        const receipt = await getTransactionReceiptWithRetry(context, hash);

        expect(receipt.status).toBe("success");

        console.log(`Transaction receipt status: ${receipt.status}`);
        console.log(`Transaction receipt logs:`, receipt.logs);

        // Check the transaction details
        const tx = await context.viem().getTransaction({ hash });
        console.log(`Transaction type: ${tx.type}`);
        console.log(`Transaction authorizationList:`, tx.authorizationList);

        // Also check the raw transaction
        console.log(`Raw transaction:`, tx);

        // Check if the delegating address now has delegated code
        const codeAtDelegator = await context.viem().getCode({
          address: delegatingAddress,
        });
        console.log(`Code at delegator address ${delegatingAddress}: ${codeAtDelegator}`);

        // Also check code at contract address for comparison
        const codeAtContract = await context.viem().getCode({
          address: contractAddress,
        });
        console.log(
          `Code at contract address ${contractAddress}: ${codeAtContract?.slice(0, 50)}...`
        );

        // EIP-7702 sets a special delegated code format: 0xef0100 + 20-byte address
        expect(codeAtDelegator).toBeTruthy();
        expect(codeAtDelegator?.startsWith("0xef0100")).toBe(true);
        expect(codeAtDelegator?.length).toBe(48); // 0x + ef0100 (6) + address (40)

        // Now check if the delegation worked
        // The storage should be in Baltathar's account context, not the contract's

        // Calculate storage slot for mapping(address => uint256) balances
        // slot = keccak256(abi.encode(targetAddress, 0))
        const storageSlot = keccak256(
          concat([
            targetAddress.toLowerCase().padEnd(66, "0") as `0x${string}`,
            numberToHex(0n, { size: 32 }),
          ])
        );

        // Check storage at the delegating EOA's address
        const storageAtDelegator = await context.viem().getStorageAt({
          address: delegatingAddress,
          slot: storageSlot,
        });

        const actualBalance = BigInt(storageAtDelegator || "0");
        console.log(`Storage at delegating address ${delegatingAddress}: ${actualBalance}`);

        // Also check the contract storage (should be 0 if delegation worked properly)
        const contractStorageBalance = await context.viem().readContract({
          address: contractAddress!,
          abi: contractAbi,
          functionName: "getBalance",
          args: [targetAddress],
        });

        console.log(`Balance in contract storage: ${contractStorageBalance}`);

        // Let's check if we can read the balance through the delegated address
        try {
          const delegatedBalance = await context.viem().readContract({
            address: delegatingAddress,
            abi: contractAbi,
            functionName: "getBalance",
            args: [targetAddress],
          });
          console.log(`Balance read through delegated address: ${delegatedBalance}`);
        } catch (error) {
          console.log(`Error reading through delegated address:`, error);
        }

        // Happy path expectations for EIP-7702
        // The storage is NOT in the delegating address, but accessed through the contract
        // The delegating address should have the delegation code
        expect(codeAtDelegator).toBeTruthy();
        expect(codeAtDelegator?.startsWith("0xef0100")).toBe(true);

        // Reading through the delegated address should return the correct balance
        const delegatedBalance = await context.viem().readContract({
          address: delegatingAddress,
          abi: contractAbi,
          functionName: "getBalance",
          args: [targetAddress],
        });

        expect(delegatedBalance).to.equal(targetBalance);
        console.log(
          `SUCCESS: EIP-7702 delegation worked! Balance ${delegatedBalance} can be read through the delegating address`
        );

        // Additional test: call incrementBalance to verify continued delegation
        const incrementData = encodeFunctionData({
          abi: contractAbi,
          functionName: "incrementBalance",
          args: [targetAddress, 500n],
        });

        // Second transaction: increment balance through the delegated address
        // We don't need to send the authorization again since it's already set
        const incrementTx = {
          to: delegatingAddress,
          data: incrementData,
          chainId: chainId,
        };

        const signedIncrement = await createViemTransaction(context, incrementTx);
        const incrementHash = await sendRawTransaction(context, signedIncrement);
        await context.createBlock();

        const incrementReceipt = await getTransactionReceiptWithRetry(context, incrementHash);
        expect(incrementReceipt.status).toBe("success");

        // Check updated balance through the delegated address
        const updatedBalance = await context.viem().readContract({
          address: delegatingAddress,
          abi: contractAbi,
          functionName: "getBalance",
          args: [targetAddress],
        });

        expect(updatedBalance).to.equal(5500n);

        console.log(`After increment: Balance is now ${updatedBalance}`);
        console.log(`EIP-7702 delegation is working correctly!`);
      },
    });

    it({
      id: "T02",
      title: "should reject EIP-7702 authorization with invalid nonce",
      test: async () => {
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);

        // Fund the EOA
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, 1000000000000000000n),
        ]);

        // Create authorization with incorrect nonce (using nonce 1 instead of 0)
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
          nonce: 1, // Wrong nonce - should be 0 for a fresh account
        });

        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: ["0x1234567890123456789012345678901234567890", 1000n],
        });

        const transaction = {
          to: delegatingEOA.address,
          data: callData,
          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
        };

        const signature = await createViemTransaction(context, transaction);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        await getTransactionReceiptWithRetry(context, hash);

        // Check that delegation did not occur due to invalid nonce
        const codeAtDelegator = await context.viem().getCode({
          address: delegatingEOA.address,
        });

        // Code should be empty since authorization failed
        expect(codeAtDelegator).toBeFalsy();
      },
    });

    it({
      id: "T03",
      title: "delegation to zero address should reset the delegation",
      test: async () => {
        // First, create a delegation
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);
        const delegatingAddress = delegatingEOA.address;
        console.log(`Created EOA for delegation: ${delegatingAddress}`);

        // Fund the delegating EOA with some balance from ALITH
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, 1000000000000000000n),
        ]);

        // Set up initial delegation
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
          nonce: 0,
        });

        console.log(
          `Authorization created for ${delegatingAddress} to delegate to ${contractAddress}`
        );
        console.log(`Authorization nonce: ${authorization.nonce}`);

        // Create the authorization list
        const authorizationList = [authorization];

        // Use the delegation ABI from helpers

        // Set balance for an arbitrary address
        const targetAddress = "0x1234567890123456789012345678901234567890" as `0x${string}`;
        const targetBalance = 1000n;

        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: [targetAddress, targetBalance],
        });

        // Create the transaction object with authorizationList
        const transaction = {
          to: delegatingAddress,
          data: callData,
          chainId: chainId,
          authorizationList,
          txnType: "eip7702" as const,
        };

        const signature = await createViemTransaction(context, transaction);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await getTransactionReceiptWithRetry(context, hash);

        expect(receipt.status).toBe("success");

        // Verify delegation is set
        const codeAfterDelegation = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(codeAfterDelegation?.startsWith("0xef0100")).toBe(true);
        console.log(`Initial delegation code: ${codeAfterDelegation}`);

        // Verify the delegated address can be called successfully
        const initialBalance = await context.viem().readContract({
          address: delegatingEOA.address,
          abi: contractAbi,
          functionName: "getBalance",
          args: ["0x1234567890123456789012345678901234567890"],
        });
        expect(initialBalance).toBe(1000n);
        console.log(`Initial balance through delegation: ${initialBalance}`);

        // Now clear the delegation by authorizing to zero address
        const clearAuthorization = await delegatingEOA.signAuthorization({
          contractAddress: "0x0000000000000000000000000000000000000000",
          chainId: chainId,
          nonce: 1, // Nonce should be incremented
        });

        // Create the authorization list
        const clearAuthorizationList = [clearAuthorization];

        const clearTransaction = {
          to: "0x0000000000000000000000000000000000000000", // any address without code work
          data: "0x",
          chainId: chainId,
          authorizationList: clearAuthorizationList,
          txnType: "eip7702" as const,
        };

        const clearSignature = await createViemTransaction(context, clearTransaction);
        const clearHash = await sendRawTransaction(context, clearSignature);
        await context.createBlock();

        const clearReceipt = await getTransactionReceiptWithRetry(context, clearHash);

        expect(clearReceipt.status).toBe("success");

        // Check that delegation should be cleared according to EIP-7702
        const codeAfterClear = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        console.log(`Code after clearing attempt: ${codeAfterClear}`);

        // According to EIP-7702, delegation to zero address should clear the code
        if (codeAfterClear === "0x" || !codeAfterClear) {
          console.log("✅ Delegation properly cleared to zero address");
          // Try to call - should fail
          await expect(
            context.viem().readContract({
              address: delegatingEOA.address,
              abi: contractAbi,
              functionName: "getBalance",
              args: ["0x1234567890123456789012345678901234567890"],
            })
          ).rejects.toThrow();
        } else {
          expect.fail("🐛 BUG: Delegation not properly cleared - code still present");
          console.log(`Code after clear: ${codeAfterClear}`);

          // Extract delegated address from the code
          if (codeAfterClear!.startsWith("0xef0100")) {
            const delegatedAddress = "0x" + codeAfterClear!.slice(8);
            console.log(`Delegated address after clear: ${delegatedAddress}`);

            // This should be zero address if clearing worked
            if (delegatedAddress === "0x0000000000000000000000000000000000000000") {
              console.log("✅ Delegation points to zero address (partial fix)");
            } else {
              expect.fail("🐛 BUG: Delegation still points to original contract");
            }
          }

          // Try to call the delegated address
          try {
            const balanceAfterClear = await context.viem().readContract({
              address: delegatingEOA.address,
              abi: contractAbi,
              functionName: "getBalance",
              args: ["0x1234567890123456789012345678901234567890"],
            });
            console.log(`🐛 BUG: Balance still accessible after clear: ${balanceAfterClear}`);
            expect.fail("🐛 BUG: Balance still accessible after clear: ${balanceAfterClear}");
          } catch {
            console.log("✅ Function calls properly fail after clearing");
          }
        }

        expect(codeAfterClear).toBeFalsy();
      },
    });

    it({
      id: "T04",
      title: "should reject authorization with mismatched chain ID",
      test: async () => {
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, 1000000000000000000n),
        ]);

        // Create authorization with wrong chain ID
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1, // Wrong chain ID (should be 1281)
          nonce: 0,
        });

        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: ["0x1234567890123456789012345678901234567890", 1000n],
        });

        const transaction = {
          to: delegatingEOA.address,
          data: callData,
          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
        };

        const signature = await createViemTransaction(context, transaction);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        await getTransactionReceiptWithRetry(context, hash);

        // Check that delegation did not occur due to chain ID mismatch
        const codeAtDelegator = await context.viem().getCode({
          address: delegatingEOA.address,
        });

        // Code should be empty since authorization failed
        expect(codeAtDelegator).toBeFalsy();
      },
    });
  },
});
