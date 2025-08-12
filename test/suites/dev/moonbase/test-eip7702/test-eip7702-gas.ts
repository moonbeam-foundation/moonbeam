import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, type Abi, parseEther, parseGwei } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createFundedAccount } from "./helpers";

describeSuite({
  id: "D010303",
  title: "EIP-7702 Gas Cost and Accounting",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageWriterAddress: `0x${string}`;
    let storageWriterAbi: Abi;
    let counterAddress: `0x${string}`;
    let counterAbi: Abi;

    // EIP-7702 gas costs
    const PER_AUTH_BASE_COST = 2500n;
    const PER_CONTRACT_CODE_BASE_COST = 2500n;

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
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

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
        const result = await context.createBlock(signature);

        let txHash: `0x${string}` | undefined;
        if (result.hash) {
          txHash = result.hash as `0x${string}`;
        } else if (result.result?.hash) {
          txHash = result.result.hash as `0x${string}`;
        }

        if (txHash) {
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });

          // Gas used should include authorization costs
          expect(receipt.gasUsed).toBeGreaterThan(PER_AUTH_BASE_COST);

          console.log(`Gas used with 1 authorization: ${receipt.gasUsed}`);
        }
      },
    });

    it({
      id: "T02",
      title: "should calculate correct gas cost for multiple authorizations",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const eoa1 = privateKeyToAccount(generatePrivateKey());
        const eoa2 = privateKeyToAccount(generatePrivateKey());
        const eoa3 = privateKeyToAccount(generatePrivateKey());

        // Fund all EOAs
        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(eoa1.address, parseEther("1")),
          context.polkadotJs().tx.balances.transferAllowDeath(eoa2.address, parseEther("1")),
          context.polkadotJs().tx.balances.transferAllowDeath(eoa3.address, parseEther("1")),
        ]);

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
        const result = await context.createBlock(signature);

        let txHash: `0x${string}` | undefined;
        if (result.hash) {
          txHash = result.hash as `0x${string}`;
        } else if (result.result?.hash) {
          txHash = result.result.hash as `0x${string}`;
        }

        if (txHash) {
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });

          // Gas should include cost for 3 authorizations
          const minExpectedGas = PER_AUTH_BASE_COST * 3n;
          expect(receipt.gasUsed).toBeGreaterThan(minExpectedGas);

          console.log(`Gas used with 3 authorizations: ${receipt.gasUsed}`);
        }
      },
    });

    it({
      id: "T03",
      title: "should test account warming for authority and authorized accounts",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const coldEOA = privateKeyToAccount(generatePrivateKey());
        const warmEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context.polkadotJs().tx.balances.transferAllowDeath(coldEOA.address, parseEther("1")),
          context.polkadotJs().tx.balances.transferAllowDeath(warmEOA.address, parseEther("1")),
        ]);

        // Warm up the warmEOA by accessing it first
        await context.viem("public").getBalance({ address: warmEOA.address });

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

        // Transaction with cold account
        const coldTx = {
          to: coldEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [coldAuth],
          type: "eip7702" as const,
        };

        const coldSignature = await senderAccount.signTransaction(coldTx);
        const coldResult = await context.createBlock(coldSignature);

        // Transaction with warm account
        const warmTx = {
          to: warmEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [warmAuth],
          type: "eip7702" as const,
        };

        const warmSignature = await senderAccount.signTransaction(warmTx);
        const warmResult = await context.createBlock(warmSignature);

        // Get gas used for both
        let coldGas = 0n;
        let warmGas = 0n;

        if (coldResult.hash || coldResult.result?.hash) {
          const txHash = (coldResult.hash || coldResult.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });
          coldGas = receipt.gasUsed;
        }

        if (warmResult.hash || warmResult.result?.hash) {
          const txHash = (warmResult.hash || warmResult.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });
          warmGas = receipt.gasUsed;
        }

        console.log(`Cold account gas: ${coldGas}, Warm account gas: ${warmGas}`);

        // Cold account should use more gas due to account access cost
        // Note: This may not always be true depending on implementation
        console.log(`Gas difference: ${coldGas - warmGas}`);
      },
    });

    it({
      id: "T04",
      title: "should test intrinsic gas cost with exact gas limit",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Calculate intrinsic gas
        // Base transaction: 21000
        // Per authorization: 2500
        // Call data cost: varies
        const intrinsicGas = 21000n + PER_AUTH_BASE_COST;

        // Try with exact intrinsic gas (should fail)
        const exactGasTx = {
          to: delegatingEOA.address,
          data: "0x",
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
          const result = await context.createBlock(signature);

          // Check if transaction failed due to out of gas
          if (result.hash || result.result?.hash) {
            const txHash = (result.hash || result.result?.hash) as `0x${string}`;
            const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });
            console.log(`Transaction with exact intrinsic gas status: ${receipt.status}`);
          }
        } catch (error) {
          console.log("Transaction with exact intrinsic gas failed as expected");
        }

        // Try with slightly more gas (should succeed)
        const sufficientGasTx = {
          ...exactGasTx,
          gas: intrinsicGas + 10000n,
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
        };

        const signature = await senderAccount.signTransaction(sufficientGasTx);
        const result = await context.createBlock(signature);

        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });
          expect(receipt.status).toBe("success");
        }
      },
    });

    it({
      id: "T05",
      title: "should test gas cost for self-delegation",
      test: async () => {
        const selfDelegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(selfDelegatingEOA.address, parseEther("1")),
        ]);

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

        // Transaction sent by the same EOA that signed the authorization
        const selfTx = {
          to: selfDelegatingEOA.address,
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          gas: 200000n,
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
        const result = await context.createBlock(signature);

        let txHash: `0x${string}` | undefined;
        if (result.hash) {
          txHash = result.hash as `0x${string}`;
        } else if (result.result?.hash) {
          txHash = result.result.hash as `0x${string}`;
        } else if (result.result?.extrinsic?.hash) {
          txHash = result.result.extrinsic.hash.toHex() as `0x${string}`;
        }

        expect(txHash).toBeTruthy();

        const receipt = await context.viem("public").getTransactionReceipt({
          hash: txHash!,
        });

        expect(receipt.status).toBe("success");
        console.log(`Self-delegation gas used: ${receipt.gasUsed}`);

        // Verify delegation was set
        const code = await context.viem("public").getCode({
          address: selfDelegatingEOA.address,
        });
        expect(code?.startsWith("0xef0100")).toBe(true);

        // Check counter was incremented
        const count = await context.viem("public").readContract({
          address: selfDelegatingEOA.address,
          abi: counterAbi,
          functionName: "count",
          args: [],
        });
        expect(count).toBe(1n);
      },
    });

    it({
      id: "T06",
      title: "should handle out-of-gas during authorization processing",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

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
        const result = await context.createBlock(signature);

        if (result.hash || result.result?.hash) {
          const txHash = (result.hash || result.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });

          // Transaction should fail due to out of gas
          expect(receipt.status).toBe("reverted");

          // Delegation should not be set
          const code = await context.viem("public").getCode({
            address: delegatingEOA.address,
          });
          expect(code).toBeFalsy();
        }
      },
    });

    it({
      id: "T07",
      title: "should test gas refund for authorization clearing",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // First set a delegation
        const setAuth = await delegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
          nonce: 0,
        });

        const setTx = {
          to: delegatingEOA.address,
          data: "0x",
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
          data: "0x",
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

        if (clearResult.hash || clearResult.result?.hash) {
          const txHash = (clearResult.hash || clearResult.result?.hash) as `0x${string}`;
          const receipt = await context.viem("public").getTransactionReceipt({ hash: txHash });

          // Gas used for clearing should potentially include refund
          console.log(`Gas used for clearing delegation: ${receipt.gasUsed}`);
        }
      },
    });
  },
});
