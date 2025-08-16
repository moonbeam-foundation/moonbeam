import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, type Abi, parseEther, parseGwei, zeroAddress } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { expectOk } from "../../../../helpers";
import { createFundedAccount } from "./helpers";

describeSuite({
  id: "D020803",
  title: "EIP-7702 Delegatecall Operations",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageWriterAddress: `0x${string}`;
    let storageWriterAbi: Abi;
    let contextCheckerAddress: `0x${string}`;
    let contextCheckerAbi: Abi;
    let callerAddress: `0x${string}`;
    let callerAbi: Abi;
    let counterAddress: `0x${string}`;
    let counterAbi: Abi;

    beforeAll(async () => {
      // Deploy test contracts
      const storageWriter = await deployCreateCompiledContract(context, "StorageWriter");
      storageWriterAddress = storageWriter.contractAddress;
      storageWriterAbi = storageWriter.abi;

      const contextChecker = await deployCreateCompiledContract(context, "ContextChecker");
      contextCheckerAddress = contextChecker.contractAddress;
      contextCheckerAbi = contextChecker.abi;

      const caller = await deployCreateCompiledContract(context, "Caller");
      callerAddress = caller.contractAddress;
      callerAbi = caller.abi;

      const counter = await deployCreateCompiledContract(context, "Counter");
      counterAddress = counter.contractAddress;
      counterAbi = counter.abi;
    });

    it({
      id: "T01",
      title: "should perform delegatecall to empty account",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);
        const emptyTarget = privateKeyToAccount(generatePrivateKey());

        // Create authorization for caller contract
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Prepare delegatecall to empty account
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "delegatecallAddress",
          args: [emptyTarget.address, "0x"],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
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

        // Get transaction receipt to check for events and status
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        // Verify transaction succeeded
        expect(receipt.status).toBe("success");

        // Verify delegation was set
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code?.startsWith("0xef0100")).toBe(true);
      },
    });

    it({
      id: "T02",
      title: "should perform delegatecall to EOA",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);
        const targetEOA = await createFundedAccount(context);

        // Create authorization
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Delegatecall to EOA
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "delegatecallAddress",
          args: [targetEOA.address, "0x"],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
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
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T03",
      title: "should perform delegatecall to contract account",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        // Create authorization for caller contract
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Delegatecall to storage writer contract
        const storeData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [1n, 42n],
        });

        // NOTE: When contract A executes delegatecall to contract B, B's code is executed
        // with contract A's storage, msg.sender and msg.value
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "delegatecallAddress",
          args: [storageWriterAddress, storeData],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,
          gas: 300000n,
          maxFeePerGas: 10_000_000_000n,
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
        expect(receipt.status).toBe("success");

        // Storage should be in the delegating EOA's context (via caller contract delegation)
        // This is complex because of double delegation - may need to check actual storage slots
      },
    });

    it({
      id: "T04",
      title: "should verify storage state after delegation",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        // Create authorization for storage writer
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Store value directly
        const callData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [5n, 100n],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [authorization],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        await expectOk(context.createBlock(signature));

        // Read the stored value
        const storedValue = await context.viem("public").readContract({
          address: delegatingEOA.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [5n],
        });

        expect(storedValue).toBe(100n);
      },
    });

    it({
      id: "T05",
      title: "should handle calls from existing contracts to delegated EOAs",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        // Delegate EOA to counter contract
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Initialize delegation
        const tx = {
          to: delegatingEOA.address,
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
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
        expect(receipt.status).toBe("success");

        // Now call the delegated EOA from another contract
        const tx2 = {
          to: callerAddress,
          data: encodeFunctionData({
            abi: callerAbi,
            functionName: "callAddress",
            args: [
              delegatingEOA.address,
              encodeFunctionData({
                abi: counterAbi,
                functionName: "increment",
                args: [],
              }),
            ],
          }),
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [authorization],
          type: "eip7702" as const,
        };

        {
          const signature = await senderAccount.signTransaction(tx2);
          const { result } = await context.createBlock(signature);

          const receipt = await context.viem("public").getTransactionReceipt({
            hash: result?.hash as `0x${string}`,
          });
          expect(receipt.status).toBe("success");
        }

        // Check counter value
        const count = await context.viem("public").readContract({
          address: delegatingEOA.address,
          abi: counterAbi,
          functionName: "count",
          args: [],
        });

        expect(count).toBe(2n); // Incremented twice
      },
    });

    it({
      id: "T06",
      title: "should handle context opcodes (ADDRESS, BALANCE, CODESIZE)",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);

        // Delegate to context checker
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const tx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`, // Empty call to establish delegation
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [authorization],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        await expectOk(context.createBlock(signature));

        // Check ADDRESS opcode
        const address = await context.viem("public").readContract({
          address: delegatingEOA.address,
          abi: contextCheckerAbi,
          functionName: "getAddress",
          args: [],
        });
        expect(address.toLowerCase()).toBe(delegatingEOA.address.toLowerCase());

        // Check BALANCE opcode
        const balance = await context.viem("public").readContract({
          address: delegatingEOA.address,
          abi: contextCheckerAbi,
          functionName: "getBalance",
          args: [],
        });
        expect(balance).toBeGreaterThan(0n);

        // Check CODESIZE opcode
        const codeSize = await context.viem("public").readContract({
          address: delegatingEOA.address,
          abi: contextCheckerAbi,
          functionName: "getCodeSize",
          args: [],
        });
        expect(codeSize).toBe(23n); // EIP-7702 delegation code size

        // Check CODEHASH opcode
        const codeHash = await context.viem("public").readContract({
          address: delegatingEOA.address,
          abi: contextCheckerAbi,
          functionName: "getCodeHash",
          args: [],
        });
        expect(codeHash).toBeTruthy();
      },
    });

    it({
      id: "T07",
      title: "should handle calls to precompile addresses",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = await createFundedAccount(context);
        const ecrecoverPrecompile = "0x0000000000000000000000000000000000000001";

        // Delegate to caller contract
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Call ecrecover precompile (with dummy data)
        const precompileData = "0x" + "00".repeat(128); // Dummy data for ecrecover

        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [ecrecoverPrecompile, precompileData],
        });

        const tx = {
          to: delegatingEOA.address,
          data: callData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
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

        // Transaction should succeed (precompile returns zero address for invalid signature)
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });
  },
});
