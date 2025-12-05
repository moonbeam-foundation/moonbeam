import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { type PublicClient, createPublicClient, webSocket } from "viem";

// Configuration for the test
const LISTEN_DURATION_MS = 60_000; // Listen for 60 seconds
const MIN_BLOCKS_EXPECTED = 3; // Minimum blocks expected in the listen period

describeSuite({
  id: "S28",
  title: "eth_subscribe newHeads - Block sequence continuity",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let client: PublicClient;
    let wsEndpoint: string;

    beforeAll(async () => {
      // Get the WebSocket endpoint from the context
      // viem transport URL might be http, convert to ws
      const httpUrl = context.viem().transport.url;
      wsEndpoint = httpUrl.replace("http://", "ws://").replace("https://", "wss://");
      log(`Using WebSocket endpoint: ${wsEndpoint}`);
    });

    it({
      id: "C100",
      title: "should deliver all canonical block headers without gaps",
      timeout: LISTEN_DURATION_MS + 30_000, // Add buffer for setup/teardown
      test: async function () {
        const transport = webSocket(wsEndpoint);
        client = createPublicClient({ transport });

        const receivedBlocks: Array<{ number: bigint; hash: string; parentHash: string }> = [];
        const skippedBlocks: Array<{ expected: bigint; received: bigint }> = [];

        log(`Starting newHeads subscription for ${LISTEN_DURATION_MS / 1000} seconds...`);

        await new Promise<void>((resolve, reject) => {
          const timeoutId = setTimeout(() => {
            unwatch();
            resolve();
          }, LISTEN_DURATION_MS);

          const unwatch = client.watchBlocks({
            onBlock: (block) => {
              const blockInfo = {
                number: block.number,
                hash: block.hash,
                parentHash: block.parentHash,
              };
              receivedBlocks.push(blockInfo);
              log(`Received block #${block.number} (${block.hash.slice(0, 10)}...)`);

              // Check for gaps with the previous block
              if (receivedBlocks.length > 1) {
                const prevBlock = receivedBlocks[receivedBlocks.length - 2];
                const expectedNumber = prevBlock.number + 1n;

                if (block.number !== expectedNumber) {
                  // Gap detected
                  skippedBlocks.push({
                    expected: expectedNumber,
                    received: block.number,
                  });
                  log(
                    `⚠️  GAP DETECTED: Expected block #${expectedNumber}, ` +
                      `received #${block.number} (skipped ${block.number - expectedNumber} blocks)`
                  );
                }
              }
            },
            onError: (error) => {
              clearTimeout(timeoutId);
              reject(error);
            },
          });
        });

        log(`\nSubscription summary:`);
        log(`  - Total blocks received: ${receivedBlocks.length}`);
        if (receivedBlocks.length > 0) {
          log(`  - First block: #${receivedBlocks[0].number}`);
          log(`  - Last block: #${receivedBlocks[receivedBlocks.length - 1].number}`);
        }

        // Verify we received enough blocks
        expect(
          receivedBlocks.length,
          `Expected at least ${MIN_BLOCKS_EXPECTED} blocks, received ${receivedBlocks.length}`
        ).toBeGreaterThanOrEqual(MIN_BLOCKS_EXPECTED);

        // Check for skipped blocks
        if (skippedBlocks.length > 0) {
          log(`\n❌ SKIPPED BLOCKS DETECTED:`);
          for (const skip of skippedBlocks) {
            const gapSize = skip.received - skip.expected;
            log(
              `  - Gap at block #${skip.expected}: jumped to #${skip.received} (${gapSize} blocks missing)`
            );
          }
        } else {
          log(`\n✓ No gaps detected in block sequence`);
        }

        // The test should fail if any blocks were skipped
        expect(
          skippedBlocks,
          `newHeads subscription skipped blocks: ${skippedBlocks
            .map((s) => `expected #${s.expected}, got #${s.received}`)
            .join("; ")}`
        ).toHaveLength(0);
      },
    });

    it({
      id: "C200",
      title: "should verify parent hashes reference previously delivered blocks",
      timeout: LISTEN_DURATION_MS + 30_000,
      test: async function () {
        const transport = webSocket(wsEndpoint);
        client = createPublicClient({ transport });

        // Track all received block hashes for parent verification
        const receivedHashes = new Set<string>();
        const receivedBlocks: Array<{ number: bigint; hash: string; parentHash: string }> = [];
        const missingParents: Array<{
          blockNumber: bigint;
          blockHash: string;
          parentHash: string;
        }> = [];
        let firstBlockNumber: bigint | null = null;

        log(`Starting newHeads subscription for ${LISTEN_DURATION_MS / 1000} seconds...`);

        await new Promise<void>((resolve, reject) => {
          const timeoutId = setTimeout(() => {
            unwatch();
            resolve();
          }, LISTEN_DURATION_MS);

          const unwatch = client.watchBlocks({
            onBlock: (block) => {
              receivedBlocks.push({
                number: block.number,
                hash: block.hash,
                parentHash: block.parentHash,
              });

              // Track first block number to skip parent check for it
              if (firstBlockNumber === null) {
                firstBlockNumber = block.number;
              }

              // For blocks after the first, verify parent hash was delivered
              // This catches the bug where blocks are skipped during reorgs
              if (block.number > firstBlockNumber && !receivedHashes.has(block.parentHash)) {
                missingParents.push({
                  blockNumber: block.number,
                  blockHash: block.hash,
                  parentHash: block.parentHash,
                });
                log(
                  `⚠️  MISSING PARENT at block #${block.number}: ` +
                    `parent ${block.parentHash.slice(0, 10)}... was never delivered`
                );
              }

              // Add this block's hash to the set of received hashes
              receivedHashes.add(block.hash);
            },
            onError: (error) => {
              clearTimeout(timeoutId);
              reject(error);
            },
          });
        });

        log(`\nSubscription summary:`);
        log(`  - Total blocks received: ${receivedBlocks.length}`);
        log(`  - Unique block hashes: ${receivedHashes.size}`);

        // Verify we received enough blocks
        expect(
          receivedBlocks.length,
          `Expected at least ${MIN_BLOCKS_EXPECTED} blocks, received ${receivedBlocks.length}`
        ).toBeGreaterThanOrEqual(MIN_BLOCKS_EXPECTED);

        // Check for missing parent blocks
        if (missingParents.length > 0) {
          log(`\n❌ MISSING PARENT BLOCKS DETECTED (blocks were skipped):`);
          for (const missing of missingParents) {
            log(
              `  - Block #${missing.blockNumber} (${missing.blockHash.slice(0, 10)}...): ` +
                `parent ${missing.parentHash.slice(0, 18)}... was never delivered`
            );
          }
        } else {
          log(`\n✓ All parent hashes reference previously delivered blocks`);
        }

        // The test should fail if any parent blocks were never delivered
        expect(
          missingParents,
          `Missing parent blocks detected: ${missingParents.length} blocks reference parents that were never delivered`
        ).toHaveLength(0);
      },
    });
  },
});
