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
      title: "should return a valid subscription ID for newHeads",
      test: async function () {
        const transport = webSocket(wsEndpoint);
        client = createPublicClient({ transport });

        const result = (await client.transport.request({
          method: "eth_subscribe",
          params: ["newHeads"],
        })) as string;

        expect(result).toBeDefined();
        expect(result.length).toBe(34); // 0x + 32 hex chars
        log(`Subscription ID: ${result}`);

        // Clean up
        await client.transport.request({
          method: "eth_unsubscribe",
          params: [result],
        });
      },
    });

    it({
      id: "C200",
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
      id: "C300",
      title: "should verify parent hash continuity in received blocks",
      timeout: LISTEN_DURATION_MS + 30_000,
      test: async function () {
        const transport = webSocket(wsEndpoint);
        client = createPublicClient({ transport });

        const receivedBlocks: Array<{ number: bigint; hash: string; parentHash: string }> = [];
        const parentHashMismatches: Array<{
          blockNumber: bigint;
          expectedParentHash: string;
          actualParentHash: string;
        }> = [];

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

              // Check parent hash continuity (only if blocks are consecutive)
              if (receivedBlocks.length > 1) {
                const prevBlock = receivedBlocks[receivedBlocks.length - 2];

                // Only check parent hash if this is the next consecutive block
                if (block.number === prevBlock.number + 1n) {
                  if (block.parentHash !== prevBlock.hash) {
                    parentHashMismatches.push({
                      blockNumber: block.number,
                      expectedParentHash: prevBlock.hash,
                      actualParentHash: block.parentHash,
                    });
                    log(
                      `⚠️  PARENT HASH MISMATCH at block #${block.number}: ` +
                        `expected ${prevBlock.hash.slice(0, 10)}..., ` +
                        `got ${block.parentHash.slice(0, 10)}...`
                    );
                  }
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

        // Verify we received enough blocks
        expect(
          receivedBlocks.length,
          `Expected at least ${MIN_BLOCKS_EXPECTED} blocks, received ${receivedBlocks.length}`
        ).toBeGreaterThanOrEqual(MIN_BLOCKS_EXPECTED);

        // Check for parent hash mismatches
        if (parentHashMismatches.length > 0) {
          log(`\n❌ PARENT HASH MISMATCHES DETECTED:`);
          for (const mismatch of parentHashMismatches) {
            log(
              `  - Block #${mismatch.blockNumber}: ` +
                `expected parent ${mismatch.expectedParentHash.slice(0, 18)}..., ` +
                `got ${mismatch.actualParentHash.slice(0, 18)}...`
            );
          }
        } else {
          log(`\n✓ Parent hash continuity verified for consecutive blocks`);
        }

        // The test should fail if any parent hash mismatches were found
        expect(
          parentHashMismatches,
          `Parent hash mismatches found in ${parentHashMismatches.length} blocks`
        ).toHaveLength(0);
      },
    });
  },
});
