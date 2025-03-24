import { describeSuite, expect, customDevRpcRequest } from "@moonwall/cli";
import { createRawTransfer, BALTATHAR_ADDRESS, BALTATHAR_PRIVATE_KEY } from "@moonwall/util";
import { parseGwei } from "viem";

describeSuite({
  id: "D011204",
  title: "Ethereum RPC - eth_pendingTransactions",
  foundationMethods: "dev",
  testCases: ({ it, context }) => {
    const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

    // Helper function to get pending transactions
    async function getPendingTransactions() {
      return await customDevRpcRequest("eth_pendingTransactions", []);
    }

    it({
      id: "T01",
      title: "should return empty array when no transactions are pending",
      test: async function () {
        // Create a block to clear any pending transactions
        await context.createBlock();

        // Check pending transactions
        const pendingTransactions = await getPendingTransactions();
        expect(pendingTransactions).toBeInstanceOf(Array);
        expect(pendingTransactions.length).toBe(0);
      },
    });

    it({
      id: "T02",
      title: "should return pending transactions when transactions are in txpool",
      test: async function () {
        // First, create a block to clear previous pending transactions
        await context.createBlock();

        // Get initial nonce
        const initialNonce = await context.viem().getTransactionCount({
          address: BALTATHAR_ADDRESS,
        });

        const readyTransactionCount = 3;
        const futureTransactionCount = 2;
        const transactions: `0x${string}`[] = [];

        // Submit regular transactions with sequential nonces
        for (let i = 0; i < readyTransactionCount; i++) {
          const currentNonce = initialNonce + i;
          const tx = await createRawTransfer(context, TEST_ACCOUNT, 1, {
            nonce: currentNonce,
            privateKey: BALTATHAR_PRIVATE_KEY,
          });
          const hash = await context.viem().sendRawTransaction({ serializedTransaction: tx });
          transactions.push(hash);
        }

        // Submit future transactions with gaps in nonces
        for (let i = 0; i < futureTransactionCount; i++) {
          // Create a gap by skipping some nonces
          const gapSize = i * 2 + 1;
          const futureNonce = initialNonce + readyTransactionCount + gapSize;
          const tx = await createRawTransfer(context, TEST_ACCOUNT, 1, {
            nonce: futureNonce,
            privateKey: BALTATHAR_PRIVATE_KEY,
          });
          const hash = await context.viem().sendRawTransaction({ serializedTransaction: tx });
          transactions.push(hash);
        }

        // Check pending transactions through RPC
        const pendingTransactions = await getPendingTransactions();

        // Verify the response
        expect(pendingTransactions).toBeInstanceOf(Array);
        expect(pendingTransactions.length).toBe(transactions.length);

        // Verify transaction hashes match what we submitted
        const pendingHashes = pendingTransactions.map((tx) => tx.hash);
        expect(pendingHashes).toEqual(expect.arrayContaining(transactions));
      },
    });

    it({
      id: "T03",
      title: "should remove transactions from pending transactions when block is created",
      test: async function () {
        // First, create a block to clear previous pending transactions
        await context.createBlock();

        // Get current nonce
        const nonce = await context.viem().getTransactionCount({
          address: BALTATHAR_ADDRESS,
        });

        // Submit a transaction
        const tx = await createRawTransfer(context, TEST_ACCOUNT, 1, {
          nonce: nonce,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        await context.viem().sendRawTransaction({ serializedTransaction: tx });

        // Check that it's in the pending transactions
        const pendingBefore = await getPendingTransactions();
        expect(pendingBefore.length).toBeGreaterThanOrEqual(1);
        const countBefore = pendingBefore.length;

        // Create a block to mine the pending transactions
        await context.createBlock();

        // Check pending transactions again
        const pendingAfter = await getPendingTransactions();

        // Verify there are fewer pending transactions after mining
        expect(pendingAfter.length).toBeLessThan(countBefore);
      },
    });

    it({
      id: "T04",
      title: "should include transaction details in pending transactions",
      test: async function () {
        // Create a block to clear any pending transactions
        await context.createBlock();

        // Get current nonce
        const nonce = await context.viem().getTransactionCount({
          address: BALTATHAR_ADDRESS,
        });

        // Submit a transaction with specific values
        const txValue = 1n;
        const tx = await createRawTransfer(context, TEST_ACCOUNT, 1, {
          nonce: nonce,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const hash = await context.viem().sendRawTransaction({ serializedTransaction: tx });

        // Get pending transactions
        const pendingTransactions = await getPendingTransactions();

        // Find our transaction
        const pendingTx = pendingTransactions.find((tx) => tx.hash === hash);
        expect(pendingTx).toBeDefined();

        // Check transaction details
        if (pendingTx) {
          expect(pendingTx.from.toLowerCase()).toBe(BALTATHAR_ADDRESS.toLowerCase());
          expect(pendingTx.to.toLowerCase()).toBe(TEST_ACCOUNT.toLowerCase());
          expect(BigInt(pendingTx.value)).toBe(txValue);
          expect(Number(pendingTx.nonce)).toBe(Number(nonce));
        }
      },
    });
  },
});
