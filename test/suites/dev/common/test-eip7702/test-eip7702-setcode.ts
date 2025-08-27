import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { sendRawTransaction } from "@moonwall/util";
import { encodeFunctionData, type Abi, parseEther, parseGwei, keccak256 } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createFundedAccount, createViemTransaction } from "./helpers";

describeSuite({
  id: "D020807",
  title: "EIP-7702 Core Set-Code Transaction Tests",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let storageWriterAddress: `0x${string}`;
    let storageWriterAbi: Abi;
    let transientStorageAddress: `0x${string}`;
    let transientStorageAbi: Abi;
    let selfDestructorAddress: `0x${string}`;
    let selfDestructorAbi: Abi;
    let contractCreatorAddress: `0x${string}`;
    let contractCreatorAbi: Abi;
    let callerAddress: `0x${string}`;
    let callerAbi: Abi;
    let reentrantCallerAddress: `0x${string}`;
    let reentrantCallerAbi: Abi;
    let chainId: number;

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

      // Deploy all test contracts
      const storageWriter = await deployCreateCompiledContract(context, "StorageWriter");
      storageWriterAddress = storageWriter.contractAddress;
      storageWriterAbi = storageWriter.abi;

      const transientStorage = await deployCreateCompiledContract(context, "TransientStorage");
      transientStorageAddress = transientStorage.contractAddress;
      transientStorageAbi = transientStorage.abi;

      const selfDestructor = await deployCreateCompiledContract(context, "SelfDestructor");
      selfDestructorAddress = selfDestructor.contractAddress;
      selfDestructorAbi = selfDestructor.abi;

      const contractCreator = await deployCreateCompiledContract(context, "ContractCreator");
      contractCreatorAddress = contractCreator.contractAddress;
      contractCreatorAbi = contractCreator.abi;

      const caller = await deployCreateCompiledContract(context, "Caller");
      callerAddress = caller.contractAddress;
      callerAbi = caller.abi;

      const reentrantCaller = await deployCreateCompiledContract(context, "ReentrantCaller");
      reentrantCallerAddress = reentrantCaller.contractAddress;
      reentrantCallerAbi = reentrantCaller.abi;
    });

    it({
      id: "T01",
      title: "should handle set-code transaction with self-authorization",
      test: async () => {
        const selfSponsor = await createFundedAccount(context);

        // Fund the self-sponsoring account
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(selfSponsor.account.address, parseEther("10")),
        ]);

        // Self-sponsor: account signs authorization and sends transaction
        const authorization = await selfSponsor.account.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 1,
        });

        const callData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [1n, 42n],
        });

        const tx = {
          to: selfSponsor.account.address, // Sending to self
          data: callData,

          nonce: 0, // First transaction from this account
          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: selfSponsor.privateKey,
        };

        const signedTx = await createViemTransaction(context, tx);
        console.log("Signed transaction:", signedTx);

        const hash = await sendRawTransaction(context, signedTx);
        console.log(`Transaction signed, sending to network...`);
        await context.createBlock();

        // Get transaction receipt to check for events and status
        const receipt = await context.viem().getTransactionReceipt({
          hash,
        });
        console.log("Transaction receipt:", receipt);

        expect(receipt.status).toBe("success");

        expect(receipt.status).toBe("success");

        // Verify delegation was set
        const code = await context.viem().getCode({
          address: selfSponsor.account.address,
        });
        expect(code).toBeDefined();
        expect(code?.startsWith("0xef0100")).toBe(true);

        // Verify storage was written
        const storedValue = await context.viem().readContract({
          address: selfSponsor.account.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [1n],
        });
        expect(storedValue).toBe(42n);
      },
    });

    it({
      id: "T02",
      title: "should execute simple SSTORE through set-code",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const callData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [100n, 999n],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash });
        expect(receipt.status).toBe("success");

        // Verify storage was written
        const storedValue = await context.viem().readContract({
          address: delegatingEOA.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [100n],
        });
        expect(storedValue).toBe(999n);
      },
    });

    it({
      id: "T03",
      title: "should handle set-code with existing storage and non-zero nonce",
      test: async () => {
        const sender = await createFundedAccount(context);
        const existingEOAPrivateKey = generatePrivateKey();
        const existingEOA = privateKeyToAccount(existingEOAPrivateKey);

        // Fund and use the account first
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(existingEOA.address, parseEther("10")),
        ]);

        // Make a transaction to increase nonce
        {
          const dummyTx = {
            to: "0x1234567890123456789012345678901234567890",
            chainId: chainId,
            privateKey: existingEOAPrivateKey,
          };

          const signature = await createViemTransaction(context, dummyTx);
          const hash = await sendRawTransaction(context, signature);
          await context.createBlock();

          const receipt = await context.viem().getTransactionReceipt({ hash });
          expect(receipt.status).toBe("success");
        }

        // Now the account has nonce = 1
        const currentNonce = await context.viem().getTransactionCount({
          address: existingEOA.address,
        });
        expect(currentNonce).toBe(1);

        // Set code with non-zero nonce account
        // In some implementations, authorization nonce might need to match account nonce
        const authNonce = await context.viem().getTransactionCount({
          address: existingEOA.address,
        });
        const authorization = await existingEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: authNonce, // Try using current account nonce for authorization
        });

        const callData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [5n, 555n],
        });

        const tx = {
          to: existingEOA.address,
          data: callData,

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash });

        expect(receipt.status).toBe("success");

        // Verify delegation was set despite non-zero account nonce
        const code = await context.viem().getCode({
          address: existingEOA.address,
        });
        expect(code).toBeDefined();
        expect(code?.startsWith("0xef0100")).toBe(true);

        // Verify storage
        const storedValue = await context.viem().readContract({
          address: existingEOA.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [5n],
        });
        expect(storedValue).toBe(555n);
      },
    });

    it({
      id: "T04",
      title: "should handle SSTORE then SLOAD in separate transactions",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        // First transaction: SSTORE
        const storeTx = {
          to: delegatingEOA.address,
          data: encodeFunctionData({
            abi: storageWriterAbi,
            functionName: "store",
            args: [20n, 200n],
          }),

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signedTx = await createViemTransaction(context, storeTx);
        const hash = await sendRawTransaction(context, signedTx);
        await context.createBlock();

        // Get transaction receipt to check for events and status
        const receipt = await context.viem().getTransactionReceipt({
          hash,
        });

        expect(receipt.status).toBe("success");

        // Second transaction: SLOAD (no authorization needed, already delegated)
        const loadTx = {
          to: delegatingEOA.address,
          data: encodeFunctionData({
            abi: storageWriterAbi,
            functionName: "load",
            args: [20n],
          }),

          chainId: chainId,
        };

        {
          const signedTx = await createViemTransaction(context, loadTx);
          const hash = await sendRawTransaction(context, signedTx);
          await context.createBlock();

          // Get transaction receipt to check for events and status
          const receipt = await context.viem().getTransactionReceipt({
            hash,
          });

          expect(receipt.status).toBe("success");
        }

        // Decode the return value from the transaction
        // The load function should return the stored value (200n)
        // Note: For view functions called via transactions, the return value might not be directly accessible
        // We can verify it through a static call instead
        const loadedValue = await context.viem().readContract({
          address: delegatingEOA.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [20n],
        });
        expect(loadedValue).toBe(200n);
      },
    });

    it({
      id: "T05",
      title: "should handle TSTORE with re-entry to TLOAD",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: transientStorageAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Store and load transient storage in same transaction
        const callData = encodeFunctionData({
          abi: transientStorageAbi,
          functionName: "storeAndLoad",
          args: [1n, 12345n],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T06",
      title: "should execute SELFDESTRUCT in delegated context",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;
        const recipient = privateKeyToAccount(generatePrivateKey());

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: selfDestructorAddress,
          chainId: chainId,
          nonce: 0,
        });

        const initialDelegatingBalance = await context.viem().getBalance({
          address: delegatingEOA.address,
        });

        const initialRecipientBalance = await context.viem().getBalance({
          address: recipient.address,
        });

        // Execute selfdestruct
        const callData = encodeFunctionData({
          abi: selfDestructorAbi,
          functionName: "destruct",
          args: [recipient.address],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash });
        expect(receipt.status).toBe("success");

        // After EIP-6780, SELFDESTRUCT only transfers balance in same transaction
        // Account should still exist with delegation
        const codeAfter = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(codeAfter?.startsWith("0xef0100")).toBe(true);

        // Check balances after SELFDESTRUCT
        const finalDelegatingBalance = await context.viem().getBalance({
          address: delegatingEOA.address,
        });
        const finalRecipientBalance = await context.viem().getBalance({
          address: recipient.address,
        });

        // The delegatingEOA is not paying for gas - senderAccount is
        // So the entire balance of delegatingEOA should be transferred to recipient
        // Note: After EIP-6780, SELFDESTRUCT only transfers balance but doesn't destroy the account

        // Assert that recipient received ALL funds from delegatingEOA
        expect(finalRecipientBalance).toBe(initialRecipientBalance + initialDelegatingBalance);

        // Assert that delegating EOA's balance is now zero (all transferred)
        expect(finalDelegatingBalance).toBe(0n);

        console.log(
          `Balance transfer: ${initialDelegatingBalance} wei (from ${delegatingEOA.address} to ${recipient.address})`
        );
      },
    });

    it({
      id: "T07",
      title: "should handle contract creation opcodes (CREATE, CREATE2)",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractCreatorAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Test CREATE opcode
        const createCallData = encodeFunctionData({
          abi: contractCreatorAbi,
          functionName: "createContract",
          args: [],
        });

        const createTx = {
          to: delegatingEOA.address,
          data: createCallData,

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signedTx = await createViemTransaction(context, createTx);
        const hash = await sendRawTransaction(context, signedTx);
        await context.createBlock();

        // Get transaction receipt to check for events and status
        const receipt = await context.viem().getTransactionReceipt({
          hash,
        });

        expect(receipt.status).toBe("success");

        // Check logs for ContractCreated event
        console.log(`Contract created via CREATE opcode`);
        // Indexed address parameter
        const createdAddress = receipt.logs[0].topics[1];
        expect(createdAddress).toBeDefined();
        // Should be a 32-byte hex string
        expect(createdAddress).toMatch(/^0x[0-9a-fA-F]{64}$/);

        // Test CREATE2 opcode
        const salt = keccak256("0x1234");
        const create2CallData = encodeFunctionData({
          abi: contractCreatorAbi,
          functionName: "createContract2",
          args: [salt],
        });

        const create2Tx = {
          to: delegatingEOA.address,
          data: create2CallData,
          chainId: chainId,
        };

        {
          const signedTx = await createViemTransaction(context, create2Tx);
          const hash = await sendRawTransaction(context, signedTx);
          await context.createBlock();

          // Get transaction receipt to check for events and status
          const receipt = await context.viem().getTransactionReceipt({
            hash,
          });

          expect(receipt.status).toBe("success");

          // Check logs for ContractCreated event from CREATE2
          const created2Address = receipt.logs[0].topics[1]; // Indexed address parameter
          expect(created2Address).toBeDefined();
          expect(created2Address).toMatch(/^0x[0-9a-fA-F]{64}$/); // Should be a 32-byte hex string
        }
      },
    });

    it({
      id: "T08",
      title: "should handle re-entry until max call stack depth",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: reentrantCallerAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Try to reach max depth (1024 in EVM)
        // We'll test with a smaller depth to avoid gas issues
        const targetDepth = 64n;

        const callData = encodeFunctionData({
          abi: reentrantCallerAbi,
          functionName: "reenter",
          args: [delegatingEOA.address, targetDepth],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,

          chainId: chainId,
          authorizationList: [authorization],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Check if transaction succeeded or failed due to stack depth
        const receipt = await context.viem().getTransactionReceipt({ hash });

        console.log(`Re-entry test status: ${receipt.status}`);

        // With depth 64, should succeed
        expect(receipt.status).toBe("success");

        // Verify the contract reached the expected depth
        // The depth state variable should show the maximum depth reached
        const maxDepthReached = await context.viem().readContract({
          address: delegatingEOA.address,
          abi: reentrantCallerAbi,
          functionName: "maxDepth",
          args: [],
        });
        expect(maxDepthReached).toBe(targetDepth);

        // The depth should be back to 0 after completion
        const currentDepth = await context.viem().readContract({
          address: delegatingEOA.address,
          abi: reentrantCallerAbi,
          functionName: "depth",
          args: [],
        });
        expect(currentDepth).toBe(0n);
      },
    });

    it({
      id: "T09",
      title: "should handle cross-delegation calls between set-code accounts",
      test: async () => {
        const sender = await createFundedAccount(context);
        const eoa1 = (await createFundedAccount(context)).account;
        const eoa2 = (await createFundedAccount(context)).account;

        // EOA1 delegates to caller contract
        const auth1 = await eoa1.signAuthorization({
          contractAddress: callerAddress,
          chainId: chainId,
          nonce: 0,
        });

        // EOA2 delegates to storage writer
        const auth2 = await eoa2.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Set up both delegations
        const setupTx = {
          to: "0x0000000000000000000000000000000000000000", // Any recipient wihout code should work
          data: "0x" as `0x${string}`,
          chainId: chainId,
          authorizationList: [auth1, auth2],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signedTx = await createViemTransaction(context, setupTx);
        const hash = await sendRawTransaction(context, signedTx);
        await context.createBlock();

        // Get transaction receipt to check for events and status
        const receipt = await context.viem().getTransactionReceipt({
          hash,
        });

        expect(receipt.status).toBe("success");

        // Now EOA1 (delegated to caller) calls EOA2 (delegated to storage writer)
        const storeData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [50n, 500n],
        });

        const crossCallData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [eoa2.address, storeData],
        });

        const crossCallTx = {
          to: eoa1.address,
          data: crossCallData,

          chainId: chainId,
        };

        {
          const signedTx = await createViemTransaction(context, crossCallTx);
          const hash = await sendRawTransaction(context, signedTx);
          await context.createBlock();

          // Get transaction receipt to check for events and status
          const receipt = await context.viem().getTransactionReceipt({
            hash,
          });

          expect(receipt.status).toBe("success");
        }

        // Verify storage was written in EOA2's context
        const storedValue = await context.viem().readContract({
          address: eoa2.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [50n],
        });
        expect(storedValue).toBe(500n);
      },
    });

    it({
      id: "T10",
      title: "should handle nested calls/delegations",
      test: async () => {
        const sender = await createFundedAccount(context);
        const eoa1 = (await createFundedAccount(context)).account;
        const eoa2 = (await createFundedAccount(context)).account;

        // Set up delegation chain:
        // EOA1 delegates to Caller contract (which can call other addresses)
        // EOA2 delegates to StorageWriter contract
        const auth1 = await eoa1.signAuthorization({
          contractAddress: callerAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth2 = await eoa2.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Set up both delegations in a single transaction
        const setupTx = {
          to: "0x0000000000000000000000000000000000000000", // Any recipient wihout code should work
          data: "0x" as `0x${string}`,
          chainId: chainId,
          authorizationList: [auth1, auth2],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signedTx = await createViemTransaction(context, setupTx);
        const hash = await sendRawTransaction(context, signedTx);
        await context.createBlock();

        // Get transaction receipt to check for events and status
        const receipt = await context.viem().getTransactionReceipt({
          hash,
        });

        expect(receipt.status).toBe("success");

        // Verify both delegations are set
        const code1 = await context.viem().getCode({ address: eoa1.address });
        const code2 = await context.viem().getCode({ address: eoa2.address });

        expect(code1?.startsWith("0xef0100")).toBe(true);
        expect(code2?.startsWith("0xef0100")).toBe(true);

        // Prepare the nested call: EOA2.store(42, 1337)
        const storeData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [42n, 1337n],
        });

        // Call EOA1 (as Caller) to call EOA2 with the store data
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [eoa2.address, storeData],
        });

        const chainCallTx = {
          to: eoa1.address,
          data: callData,

          chainId: chainId,
        };

        {
          const signedTx = await createViemTransaction(context, chainCallTx);
          const hash = await sendRawTransaction(context, signedTx);
          await context.createBlock();

          // Get transaction receipt to check for events and status
          const receipt = await context.viem().getTransactionReceipt({
            hash,
          });

          expect(receipt.status).toBe("success");
        }

        // Verify that storage was written in EOA2's context
        // This proves that EOA2 executed StorageWriter code, not followed another chain
        const storedValue = await context.viem().readContract({
          address: eoa2.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [42n],
        });
        expect(storedValue).toBe(1337n);

        // Also verify that EOA1 doesn't have this storage
        // EOA1 is delegated to Caller, not StorageWriter, so trying to call
        // StorageWriter functions on it should fail
        try {
          await context.viem().readContract({
            address: eoa1.address,
            abi: storageWriterAbi,
            functionName: "load",
            args: [42n],
          });
          // If we get here, the test should fail
          expect(true).toBe(false);
        } catch (error) {
          // Expected to fail since EOA1 has Caller code, not StorageWriter
          expect(error).toBeDefined();
        }

        console.log("Verified: Delegated calls do not follow chains");
      },
    });

    it({
      id: "T11",
      title: "should handle multiple authorizations in single transaction",
      test: async () => {
        const sender = await createFundedAccount(context);
        const eoa1 = (await createFundedAccount(context)).account;
        const eoa2 = (await createFundedAccount(context)).account;
        const eoa3 = (await createFundedAccount(context)).account;
        const eoa4 = (await createFundedAccount(context)).account;

        // Create multiple authorizations to different contracts
        const auth1 = await eoa1.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth2 = await eoa2.signAuthorization({
          contractAddress: callerAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth3 = await eoa3.signAuthorization({
          contractAddress: transientStorageAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth4 = await eoa4.signAuthorization({
          contractAddress: contractCreatorAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Send transaction with all authorizations
        const tx = {
          to: eoa1.address,
          data: encodeFunctionData({
            abi: storageWriterAbi,
            functionName: "store",
            args: [1n, 100n],
          }),

          chainId: chainId,
          authorizationList: [auth1, auth2, auth3, auth4],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash });
        expect(receipt.status).toBe("success");

        // Verify all delegations were set
        const code1 = await context.viem().getCode({ address: eoa1.address });
        const code2 = await context.viem().getCode({ address: eoa2.address });
        const code3 = await context.viem().getCode({ address: eoa3.address });
        const code4 = await context.viem().getCode({ address: eoa4.address });

        expect(code1?.startsWith("0xef0100")).toBe(true);
        expect(code2?.startsWith("0xef0100")).toBe(true);
        expect(code3?.startsWith("0xef0100")).toBe(true);
        expect(code4?.startsWith("0xef0100")).toBe(true);

        // Verify the actual call succeeded
        const storedValue = await context.viem().readContract({
          address: eoa1.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [1n],
        });
        expect(storedValue).toBe(100n);
      },
    });
  },
});
