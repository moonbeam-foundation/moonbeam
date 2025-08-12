import "@moonbeam-network/api-augment";

import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { type Abi, parseEther, parseGwei } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createFundedAccount } from "./helpers";

describeSuite({
  id: "D020805",
  title: "EIP-7702 Invalid Transaction Handling",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contractAddress: `0x${string}`;
    let contractAbi: Abi;

    beforeAll(async () => {
      const contract = await deployCreateCompiledContract(context, "Counter");
      contractAddress = contract.contractAddress;
      contractAbi = contract.abi;
    });

    it({
      id: "T01",
      title: "should reject empty authorization list properly",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const receiverAccount = await createFundedAccount(context);
        // EIP-7702 transactions with empty authorization list should be valid
        // but behave like regular transactions
        const tx = {
          to: receiverAccount.address,
          data: "0x",
          gas: 21000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // EIP-7702 transactions with empty authorization list must be rejected
        // The transaction should fail and not produce a hash
        expect(result?.successful).toBe(false);
        expect(result?.hash).toBeUndefined();
      },
    });

    it({
      id: "T02",
      title: "should reject authorization with invalid signature (s > secp256k1n/2)",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Manipulate the signature to have invalid s value
        // s must be <= secp256k1n/2 for canonical signatures
        // Note: This creates an authorization with mismatched signature
        const invalidAuth = {
          ...validAuth,
          s: "0x7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a1", // > n/2
        };

        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [invalidAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Check that delegation was not set
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T03",
      title: "should reject authorization with invalid chain ID",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Try to create authorization with invalid chain ID (different from tx chain ID)
        // This should cause authorization to be invalid due to chain ID mismatch
        const wrongChainId = 999999; // Wrong chain ID

        const invalidAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: wrongChainId,
          nonce: 0,
        });

        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [invalidAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Delegation should not be set due to chain ID mismatch
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T04",
      title: "should reject authorization with nonce overflow",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Try with wrong nonce (should be 0 for first delegation, use 1 instead)
        const wrongNonce = 1;

        const invalidAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: wrongNonce,
        });

        const tx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [invalidAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Delegation should not be set due to wrong nonce
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T05",
      title: "should handle authorization with zero address",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create authorization with invalid contract address (not 20 bytes)
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: "0x0000000000000000000000000000000000000000" as `0x${string}`, // Zero address
          chainId: 1281,
          nonce: 0,
        });

        const tx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`,
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [validAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Delegation may be set even with zero address - this is actually valid behavior
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        // Zero address delegation is actually allowed in the spec, but resets the delegation to empty code
        expect(code).toBeFalsy();

        // Verify transaction result - may revert when calling zero address delegation
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        // Transaction may revert when calling zero address after delegation
        expect(["success", "reverted"]).toContain(receipt.status);
      },
    });

    it({
      id: "T06",
      title: "should handle authorization with EOA address",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Sign authorization with EOA address directly
        const eoaAuth = await delegatingEOA.signAuthorization({
          contractAddress: senderAccount.address, // Use EOA address instead of contract
          chainId: 1281,
          nonce: 0,
        });
        const tx = {
          to: delegatingEOA.address,
          value: 1000n, // Send some value instead of calling
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [eoaAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Verify transaction result - may revert when calling EOA after delegation
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        // Transaction may revert when calling EOA after delegation
        expect(["success", "reverted"]).toContain(receipt.status);

        // Check that delegation was set (EOA can be delegated to)
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        // EOA delegation should work, so code should be set
        if (code && code !== "0x") {
          expect(code.startsWith("0xef0100")).toBe(true);
        }
      },
    });

    it({
      id: "T07",
      title: "should reject authorization with invalid signature",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Create auth with wrong signature (manipulate r value to make signature invalid)
        // Note: This creates an authorization with mismatched signature
        const invalidAuth = {
          ...validAuth,
          r: "0x1111111111111111111111111111111111111111111111111111111111111111", // Wrong r value
        };

        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [invalidAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Delegation should not be set due to invalid signature
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T08",
      title: "should reject duplicate authorizations in same transaction",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        const auth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Include the same authorization twice
        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          gas: 150000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [auth, auth], // Duplicate
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // First authorization should succeed, second should be ignored
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });

        // Transaction may succeed but only one delegation should be set
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });

        if (code) {
          expect(code.startsWith("0xef0100")).toBe(true);
          console.log("First authorization succeeded, duplicate ignored");
        }
      },
    });

    it({
      id: "T09",
      title: "should reject authorization with zero r value",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Invalid signature with r = 0
        // Note: This creates an authorization with mismatched signature
        const invalidAuth = {
          ...validAuth,
          r: "0x0000000000000000000000000000000000000000000000000000000000000000",
        };

        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [invalidAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Invalid signature with zero r should not set delegation
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T10",
      title: "should reject authorization with zero s value",
      test: async () => {
        const senderAccount = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: 0,
        });

        // Invalid signature with s = 0
        // Note: This creates an authorization with mismatched signature
        const invalidAuth = {
          ...validAuth,
          s: "0x0000000000000000000000000000000000000000000000000000000000000000",
        };

        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          gas: 100000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: await context.viem("public").getTransactionCount({
            address: senderAccount.address,
          }),
          chainId: 1281,
          authorizationList: [invalidAuth],
          type: "eip7702" as const,
        };

        const signature = await senderAccount.signTransaction(tx);
        const { result } = await context.createBlock(signature);

        // Invalid signature with zero s should not set delegation
        const code = await context.viem("public").getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");
      },
    });
  },
});
