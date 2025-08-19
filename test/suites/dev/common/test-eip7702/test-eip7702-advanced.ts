import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, decodeFunctionResult, type Abi, parseEther, parseGwei } from "viem";
import { expectOk } from "../../../../helpers";
import { createFundedAccount } from "./helpers";

describeSuite({
  id: "D020802",
  title: "EIP-7702 Advanced Pointer and Context Tests",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageWriterAddress: `0x${string}`;
    let storageWriterAbi: Abi;
    let contextCheckerAddress: `0x${string}`;
    let contextCheckerAbi: Abi;
    let callerAddress: `0x${string}`;
    let callerAbi: Abi;
    let storageModifierAddress: `0x${string}`;
    let storageModifierAbi: Abi;
    let ethReceiverAddress: `0x${string}`;
    let ethReceiverAbi: Abi;
    let chainId: number;

    // Precompile addresses
    const ecrecoverPrecompile = "0x0000000000000000000000000000000000000001";
    const sha256Precompile = "0x0000000000000000000000000000000000000002";
    const ripemd160Precompile = "0x0000000000000000000000000000000000000003";
    const identityPrecompile = "0x0000000000000000000000000000000000000004";

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

      const storageWriter = await deployCreateCompiledContract(context, "StorageWriter");
      storageWriterAddress = storageWriter.contractAddress;
      storageWriterAbi = storageWriter.abi;

      const contextChecker = await deployCreateCompiledContract(context, "ContextChecker");
      contextCheckerAddress = contextChecker.contractAddress;
      contextCheckerAbi = contextChecker.abi;

      const caller = await deployCreateCompiledContract(context, "Caller");
      callerAddress = caller.contractAddress;
      callerAbi = caller.abi;

      const storageModifier = await deployCreateCompiledContract(context, "StorageModifier");
      storageModifierAddress = storageModifier.contractAddress;
      storageModifierAbi = storageModifier.abi;

      const ethReceiver = await deployCreateCompiledContract(context, "EthReceiver");
      ethReceiverAddress = ethReceiver.contractAddress;
      ethReceiverAbi = ethReceiver.abi;
    });

    it({
      id: "T01",
      title: "should handle pointer chain with multiple authorization tuples",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        // Create a chain: EOA1 -> Contract1 -> EOA2 -> Contract2 -> EOA3 -> Contract3
        const eoa1 = await createFundedAccount(context);
        const eoa2 = await createFundedAccount(context);
        const eoa3 = await createFundedAccount(context);

        // Create pointer chain
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

        const auth3 = await eoa3.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Set up all pointers in one transaction
        const setupTx = {
          to: eoa1.address,
          data: "0x" as `0x${string}`,
          gas: 300000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth1, auth2, auth3],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Now test pointer chain: EOA1 calls EOA2
        const storeData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [10n, 100n],
        });

        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [eoa2.address, storeData],
        });

        const chainTx = {
          to: eoa1.address,
          data: callData,
          gas: 400000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const chainSignature = await senderAccount.signTransaction(chainTx);
        await expectOk(context.createBlock(chainSignature));

        // Verify storage in EOA2's context
        const storedValue = await context.viem("public").readContract({
          address: eoa2.address,
          abi: storageWriterAbi,
          functionName: "load",
          args: [10n],
        });
        expect(storedValue).toBe(100n);
      },
    });

    it({
      id: "T02",
      title: "should handle delegation-to-delegation calls",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const eoa1 = await createFundedAccount(context);
        const eoa2 = await createFundedAccount(context);

        // Both pointers delegate to caller contract
        const auth1 = await eoa1.signAuthorization({
          contractAddress: eoa2.address,
          chainId: chainId,
          nonce: 0,
        });

        const auth2 = await eoa2.signAuthorization({
          contractAddress: callerAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: eoa1.address,
          data: "0x" as `0x${string}`,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth1, auth2],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // eoa1 calls eoa2
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [eoa2.address, "0x"],
        });

        const pointerTx = {
          to: eoa1.address,
          data: callData,
          gas: 300000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const pointerSignature = await senderAccount.signTransaction(pointerTx);
        const { result } = await context.createBlock(pointerSignature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        // Execution must not follow delegation chains.
        expect(receipt.status).toBe("reverted");
      },
    });

    it({
      id: "T03",
      title: "should test context opcodes with pointers (BALANCE, CODESIZE, etc.)",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = await createFundedAccount(context);

        const auth = await pointer.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Test ADDRESS opcode - should return pointer's address
        const address = await context.viem("public").readContract({
          address: pointer.address,
          abi: contextCheckerAbi,
          functionName: "getAddress",
          args: [],
        });
        expect(address.toLowerCase()).toBe(pointer.address.toLowerCase());

        // Test BALANCE opcode - should return pointer's balance
        const balance = await context.viem("public").readContract({
          address: pointer.address,
          abi: contextCheckerAbi,
          functionName: "getBalance",
          args: [],
        });
        expect(balance).toBeGreaterThan(0n);

        // Test CODESIZE opcode - should return delegation code size (23 bytes)
        const codeSize = await context.viem("public").readContract({
          address: pointer.address,
          abi: contextCheckerAbi,
          functionName: "getCodeSize",
          args: [],
        });
        expect(codeSize).toBe(23n);

        // Test CODEHASH opcode
        const codeHash = await context.viem("public").readContract({
          address: pointer.address,
          abi: contextCheckerAbi,
          functionName: "getCodeHash",
          args: [],
        });
        expect(codeHash).toBeTruthy();
      },
    });

    it({
      id: "T04",
      title: "should test call to precompile in pointer context",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = await createFundedAccount(context);

        const auth = await pointer.signAuthorization({
          contractAddress: callerAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Call identity precompile through pointer
        const testData = "0x48656c6c6f20576f726c64"; // "Hello World" in hex
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [identityPrecompile, testData],
        });

        const precompileTx = {
          to: pointer.address,
          data: callData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const precompileSignature = await senderAccount.signTransaction(precompileTx);
        const { result } = await context.createBlock(precompileSignature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T05",
      title: "should test gas difference between pointer and direct calls",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = await createFundedAccount(context);

        // Set up pointer
        const auth = await pointer.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Call through pointer
        const storeData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [1n, 100n],
        });

        const pointerCallTx = {
          to: pointer.address,
          data: storeData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const pointerSignature = await senderAccount.signTransaction(pointerCallTx);
        const pointerResult = await context.createBlock(pointerSignature);

        let pointerGas = 0n;
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: pointerResult.result?.hash as `0x${string}`,
        });
        pointerGas = receipt.gasUsed;

        // Direct call to contract
        const directCallTx = {
          to: storageWriterAddress,
          data: encodeFunctionData({
            abi: storageWriterAbi,
            functionName: "store",
            args: [2n, 200n],
          }),
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const directSignature = await senderAccount.signTransaction(directCallTx);
        const directResult = await context.createBlock(directSignature);

        let directGas = 0n;
        const receipt2 = await context.viem("public").getTransactionReceipt({
          hash: directResult.result?.hash as `0x${string}`,
        });
        directGas = receipt2.gasUsed;

        console.log(`Pointer call gas: ${pointerGas}, Direct call gas: ${directGas}`);
        console.log(`Gas difference: ${pointerGas - directGas}`);

        // Pointer call should use the same gas as delegation call
        expect(pointerGas).toEqual(directGas);
      },
    });

    it({
      id: "T06",
      title: "should test static context preservation through pointers",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = await createFundedAccount(context);

        const auth = await pointer.signAuthorization({
          contractAddress: callerAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Try to make a static call that should fail if it tries to modify state
        const storeData = encodeFunctionData({
          abi: storageWriterAbi,
          functionName: "store",
          args: [1n, 100n],
        });

        const staticCallData = encodeFunctionData({
          abi: callerAbi,
          functionName: "staticcallAddress",
          args: [storageWriterAddress, storeData],
        });

        const returnData = await context.viem("public").call({
          to: pointer.address,
          data: staticCallData,
        });

        // Decode the return data to verify the static call returned false
        const [success] = decodeFunctionResult({
          abi: callerAbi,
          functionName: "staticcallAddress",
          data: returnData.data!,
        }) as [boolean, `0x${string}`];

        expect(success).toBe(false);
      },
    });

    it({
      id: "T07",
      title: "should test pointer reverts and error propagation",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = await createFundedAccount(context);

        const auth = await pointer.signAuthorization({
          contractAddress: storageModifierAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Set the contract to revert
        const setRevertTx = {
          to: pointer.address,
          data: encodeFunctionData({
            abi: storageModifierAbi,
            functionName: "setShouldRevert",
            args: [true],
          }),
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const setRevertSignature = await senderAccount.signTransaction(setRevertTx);
        await expectOk(context.createBlock(setRevertSignature));

        // Now try to set value which should revert
        const revertTx = {
          to: pointer.address,
          data: encodeFunctionData({
            abi: storageModifierAbi,
            functionName: "setValue",
            args: [1n, 100n],
          }),
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const revertSignature = await senderAccount.signTransaction(revertTx);
        const { result } = await context.createBlock(revertSignature);

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("reverted");
      },
    });

    it({
      id: "T08",
      title: "should test double authorization (last authorization wins)",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const doubleAuth = await createFundedAccount(context);

        // Create two authorizations for the same EOA
        const auth1 = await doubleAuth.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: chainId,
          nonce: 0,
        });

        const auth2 = await doubleAuth.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: chainId,
          nonce: 1,
        });

        // Send both authorizations - last one should win
        const tx = {
          to: doubleAuth.address,
          data: "0x",
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth1, auth2],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        await expectOk(context.createBlock(signature));

        // Check which delegation is active - should be contextChecker (last one)
        const address = await context.viem("public").readContract({
          address: doubleAuth.address,
          abi: contextCheckerAbi,
          functionName: "getAddress",
          args: [],
        });

        expect(address.toLowerCase()).toBe(doubleAuth.address.toLowerCase());
        console.log("Last authorization (contextChecker) is active");
      },
    });

    it({
      id: "T09",
      title: "should test pointer with ETH transfers",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = await createFundedAccount(context);

        const auth = await pointer.signAuthorization({
          contractAddress: ethReceiverAddress,
          chainId: chainId,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Get initial balances
        const initialSenderBalance = await context.viem("public").getBalance({
          address: senderAccount.address,
        });
        const initialPointerBalance = await context.viem("public").getBalance({
          address: pointer.address,
        });

        // Send ETH to the pointer (which delegates to EthReceiver)
        const sendEthTx = {
          to: pointer.address,
          value: parseEther("0.5"),
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const sendEthSignature = await senderAccount.signTransaction(sendEthTx);
        await expectOk(context.createBlock(sendEthSignature));

        // Check balance after ETH transfer
        const balanceAfterDeposit = await context.viem("public").getBalance({
          address: pointer.address,
        });
        expect(balanceAfterDeposit).toBe(initialPointerBalance + parseEther("0.5"));

        // Check deposit was recorded
        const deposit = await context.viem("public").readContract({
          address: pointer.address,
          abi: ethReceiverAbi,
          functionName: "deposits",
          args: [senderAccount.address],
        });
        expect(deposit).toBe(parseEther("0.5"));

        // Withdraw the ETH
        const withdrawTx = {
          to: pointer.address,
          data: encodeFunctionData({
            abi: ethReceiverAbi,
            functionName: "withdraw",
            args: [],
          }),
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: chainId,
        };

        const withdrawSignature = await senderAccount.signTransaction(withdrawTx);
        await expectOk(context.createBlock(withdrawSignature));

        // Check balance after withdrawal
        const balanceAfterWithdrawal = await context.viem("public").getBalance({
          address: pointer.address,
        });
        expect(balanceAfterWithdrawal).toBe(initialPointerBalance);

        // Check deposit was cleared
        const depositAfter = await context.viem("public").readContract({
          address: pointer.address,
          abi: ethReceiverAbi,
          functionName: "deposits",
          args: [senderAccount.address],
        });
        expect(depositAfter).toBe(0n);

        // Check sender's final balance (should be less than initial due to gas costs and the ETH that was withdrawn back)
        const finalSenderBalance = await context.viem("public").getBalance({
          address: senderAccount.address,
        });
        expect(finalSenderBalance).toBeLessThan(initialSenderBalance);
      },
    });
  },
});
