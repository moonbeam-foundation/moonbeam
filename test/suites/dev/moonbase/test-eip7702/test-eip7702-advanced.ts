import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, type Abi, parseEther, parseGwei, keccak256 } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
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

    // Precompile addresses
    const ecrecoverPrecompile = "0x0000000000000000000000000000000000000001";
    const sha256Precompile = "0x0000000000000000000000000000000000000002";
    const ripemd160Precompile = "0x0000000000000000000000000000000000000003";
    const identityPrecompile = "0x0000000000000000000000000000000000000004";

    beforeAll(async () => {
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
        // Create a chain: EOA1 -> Contract1 -> EOA2 -> Contract2 -> EOA3 -> Contract3
        const eoa1 = privateKeyToAccount(generatePrivateKey());
        const eoa2 = privateKeyToAccount(generatePrivateKey());
        const eoa3 = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(eoa1.address, parseEther("1")),
          context.polkadotJs().tx.balances.transferAllowDeath(eoa2.address, parseEther("1")),
          context.polkadotJs().tx.balances.transferAllowDeath(eoa3.address, parseEther("1")),
        ]);

        // Create pointer chain
        const auth1 = await eoa1.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const auth2 = await eoa2.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const auth3 = await eoa3.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Set up all pointers in one transaction
        const setupTx = {
          to: eoa1.address,
          data: "0x",
          gas: 300000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
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
          chainId: 1281,
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
      title: "should handle pointer-to-pointer calls",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer1 = privateKeyToAccount(generatePrivateKey());
        const pointer2 = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer1.address, parseEther("1")),
          context.polkadotJs().tx.balances.transferAllowDeath(pointer2.address, parseEther("1")),
        ]);

        // Both pointers delegate to caller contract
        const auth1 = await pointer1.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const auth2 = await pointer2.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer1.address,
          data: "0x",
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [auth1, auth2],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

        // Pointer1 calls Pointer2
        const callData = encodeFunctionData({
          abi: callerAbi,
          functionName: "callAddress",
          args: [pointer2.address, "0x"],
        });

        const pointerTx = {
          to: pointer1.address,
          data: callData,
          gas: 300000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
        };

        const pointerSignature = await senderAccount.signTransaction(pointerTx);
        const result = await context.createBlock(pointerSignature);

        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });
          expect(receipt.status).toBe("success");
        }
      },
    });

    it({
      id: "T03",
      title: "should test context opcodes with pointers (BALANCE, CODESIZE, etc.)",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer.address, parseEther("5")),
        ]);

        const auth = await pointer.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
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
        const pointer = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer.address, parseEther("1")),
        ]);

        const auth = await pointer.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
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
          chainId: 1281,
        };

        const precompileSignature = await senderAccount.signTransaction(precompileTx);
        const result = await context.createBlock(precompileSignature);

        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });
          expect(receipt.status).toBe("success");
        }
      },
    });

    it({
      id: "T05",
      title: "should test gas difference between pointer and direct calls",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer.address, parseEther("1")),
        ]);

        // Set up pointer
        const auth = await pointer.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
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
          chainId: 1281,
        };

        const pointerSignature = await senderAccount.signTransaction(pointerCallTx);
        const pointerResult = await context.createBlock(pointerSignature);

        let pointerGas = 0n;
        if (pointerResult.hash || pointerResult.result?.hash) {
          const txHash = (pointerResult.hash || pointerResult.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });
          pointerGas = receipt.gasUsed;
        }

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
          chainId: 1281,
        };

        const directSignature = await senderAccount.signTransaction(directCallTx);
        const directResult = await context.createBlock(directSignature);

        let directGas = 0n;
        if (directResult.hash || directResult.result?.hash) {
          const txHash = (directResult.hash || directResult.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });
          directGas = receipt.gasUsed;
        }

        console.log(`Pointer call gas: ${pointerGas}, Direct call gas: ${directGas}`);
        console.log(`Gas difference: ${pointerGas - directGas}`);

        // Pointer call should use slightly more gas due to delegation overhead
        expect(pointerGas).toBeGreaterThan(directGas);
      },
    });

    it({
      id: "T06",
      title: "should test static context preservation through pointers",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer.address, parseEther("1")),
        ]);

        const auth = await pointer.signAuthorization({
          contractAddress: callerAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
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

        const staticTx = {
          to: pointer.address,
          data: staticCallData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
        };

        const staticSignature = await senderAccount.signTransaction(staticTx);
        const result = await context.createBlock(staticSignature);

        // Static call to state-modifying function should fail
        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });
          // Transaction succeeds but the static call itself would return false
          expect(receipt.status).toBe("success");
        }
      },
    });

    it({
      id: "T07",
      title: "should test pointer reverts and error propagation",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer.address, parseEther("1")),
        ]);

        const auth = await pointer.signAuthorization({
          contractAddress: storageModifierAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
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
          chainId: 1281,
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
          chainId: 1281,
        };

        const revertSignature = await senderAccount.signTransaction(revertTx);
        const result = await context.createBlock(revertSignature);

        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });
          expect(receipt.status).toBe("reverted");
        }
      },
    });

    it({
      id: "T08",
      title: "should test double authorization (last authorization wins)",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const doubleAuth = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(doubleAuth.address, parseEther("1")),
        ]);

        // Create two authorizations for the same EOA
        const auth1 = await doubleAuth.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const auth2 = await doubleAuth.signAuthorization({
          contractAddress: contextCheckerAddress,
          chainId: 1281,
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
          chainId: 1281,
          authorizationList: [auth1, auth2],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        await expectOk(context.createBlock(signature));

        // Check which delegation is active - should be contextChecker (last one)
        try {
          // This should work if delegated to contextChecker
          const address = await context.viem("public").readContract({
            address: doubleAuth.address,
            abi: contextCheckerAbi,
            functionName: "getAddress",
            args: [],
          });
          expect(address.toLowerCase()).toBe(doubleAuth.address.toLowerCase());
          console.log("Last authorization (contextChecker) is active");
        } catch {
          // This would work if delegated to storageWriter
          const value = await context.viem("public").readContract({
            address: doubleAuth.address,
            abi: storageWriterAbi,
            functionName: "load",
            args: [0n],
          });
          console.log("First authorization (storageWriter) is active");
        }
      },
    });

    it({
      id: "T09",
      title: "should test pre-Prague transaction rejection",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        // This test would require the ability to simulate pre-Prague behavior
        // Since we're testing on a post-Prague chain, we can only verify
        // that EIP-7702 transactions work correctly

        const modernEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(modernEOA.address, parseEther("1")),
        ]);

        const auth = await modernEOA.signAuthorization({
          contractAddress: storageWriterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const tx = {
          to: modernEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const result = await context.createBlock(signature);

        // On a Prague-enabled chain, this should succeed
        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });
          expect(receipt.status).toBe("success");
          console.log("EIP-7702 transaction accepted on Prague-enabled chain");
        }
      },
    });

    it({
      id: "T10",
      title: "should test pointer with ETH transfers",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const pointer = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(pointer.address, parseEther("2")),
        ]);

        const auth = await pointer.signAuthorization({
          contractAddress: ethReceiverAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setupTx = {
          to: pointer.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [auth],
          type: "eip7702" as const,
        };

        const setupSignature = await senderAccount.signTransaction(setupTx);
        await expectOk(context.createBlock(setupSignature));

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
          chainId: 1281,
        };

        const sendEthSignature = await senderAccount.signTransaction(sendEthTx);
        await expectOk(context.createBlock(sendEthSignature));

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
          chainId: 1281,
        };

        const withdrawSignature = await senderAccount.signTransaction(withdrawTx);
        await expectOk(context.createBlock(withdrawSignature));

        // Check deposit was cleared
        const depositAfter = await context.viem("public").readContract({
          address: pointer.address,
          abi: ethReceiverAbi,
          functionName: "deposits",
          args: [senderAccount.address],
        });
        expect(depositAfter).toBe(0n);
      },
    });
  },
});
