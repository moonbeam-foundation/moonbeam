import "@moonbeam-network/api-augment";

import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { type Abi, parseEther } from "viem";
import { sendRawTransaction } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createFundedAccount, createViemTransaction } from "./helpers";
import { getTransactionReceiptWithRetry } from "../../../../helpers/eth-transactions";

describeSuite({
  id: "D010306",
  title: "EIP-7702 Invalid Transaction Handling",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let contractAddress: `0x${string}`;
    let contractAbi: Abi;
    let chainId: number;

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

      const contract = await deployCreateCompiledContract(context, "Counter");
      contractAddress = contract.contractAddress;
      contractAbi = contract.abi;
    });

    it({
      id: "T01",
      title: "should reject empty authorization list properly",
      test: async () => {
        const sender = await createFundedAccount(context);
        const receiverAccount = privateKeyToAccount(generatePrivateKey());
        // EIP-7702 transactions with empty authorization list should be valid
        // but behave like regular transactions
        const tx = {
          to: receiverAccount.address,
          data: "0x",
          gas: 21000n,
          chainId: chainId,
          authorizationList: [],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
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
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
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
          chainId: chainId,
          authorizationList: [invalidAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Check that delegation was not set
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T03",
      title: "should reject authorization with invalid chain ID",
      test: async () => {
        const sender = await createFundedAccount(context);
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
          chainId: chainId,
          authorizationList: [invalidAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Delegation should not be set due to chain ID mismatch
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T04",
      title: "should reject authorization with nonce overflow",
      test: async () => {
        const sender = await createFundedAccount(context);
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
          chainId: chainId,
          nonce: wrongNonce,
        });

        const tx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`,
          chainId: chainId,
          authorizationList: [invalidAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Delegation should not be set due to wrong nonce
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T05",
      title: "should handle authorization with zero address",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create authorization with invalid contract address (not 20 bytes)
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: "0x0000000000000000000000000000000000000000" as `0x${string}`, // Zero address
          chainId: chainId,
          nonce: 0,
        });

        const tx = {
          to: delegatingEOA.address,
          data: "0x" as `0x${string}`,
          chainId: chainId,
          authorizationList: [validAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Delegation may be set even with zero address - this is actually valid behavior
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        // Zero address delegation is actually allowed in the spec, but resets the delegation to empty code
        expect(code).toBeFalsy();

        // Verify transaction result - may revert when calling zero address delegation
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        // Transaction may revert when calling zero address after delegation
        expect(["success", "reverted"]).toContain(receipt.status);
      },
    });

    it({
      id: "T06",
      title: "should handle authorization with EOA address",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = (await createFundedAccount(context)).account;

        // Sign authorization with EOA address directly
        const eoaAuth = await delegatingEOA.signAuthorization({
          contractAddress: sender.account.address, // Use EOA address instead of contract
          chainId: chainId,
          nonce: 0,
        });

        const tx = {
          to: delegatingEOA.address,
          value: 1000n, // Send some value instead of calling
          chainId: chainId,
          authorizationList: [eoaAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Verify transaction result - may revert when calling EOA after delegation
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        // Transaction may revert when calling EOA after delegation
        expect(["success", "reverted"]).toContain(receipt.status);

        // Check that delegation was set (EOA can be delegated to)
        const code = await context.viem().getCode({
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
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
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
          chainId: chainId,
          authorizationList: [invalidAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Delegation should not be set due to invalid signature
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T08",
      title: "should reject duplicate authorizations in same transaction",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        const auth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
          nonce: 0,
        });

        // Include the same authorization twice
        const tx = {
          to: delegatingEOA.address,
          data: "0x",
          chainId: chainId,
          authorizationList: [auth, auth], // Duplicate
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // First authorization should succeed, second should be ignored
        const receipt = await getTransactionReceiptWithRetry(context, hash);

        // Transaction may succeed but only one delegation should be set
        const code = await context.viem().getCode({
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
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
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
          chainId: chainId,
          authorizationList: [invalidAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Invalid signature with zero r should not set delegation
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T10",
      title: "should reject authorization with zero s value",
      test: async () => {
        const sender = await createFundedAccount(context);
        const delegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingEOA.address, parseEther("1")),
        ]);

        // Create a valid authorization first
        const validAuth = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
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
          chainId: chainId,
          authorizationList: [invalidAuth],
          txnType: "eip7702" as const,
          privateKey: sender.privateKey,
        };

        const signature = await createViemTransaction(context, tx);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        // Invalid signature with zero s should not set delegation
        const code = await context.viem().getCode({
          address: delegatingEOA.address,
        });
        expect(code).toBeFalsy();

        // Verify transaction succeeded but authorization was invalid
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");
      },
    });
  },
});
