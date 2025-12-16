import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { type PublicClient, createPublicClient, webSocket } from "viem";
import type { ApiPromise } from "@polkadot/api";

// Configuration for the test
// Longer duration increases the chance of observing a reorg on a live network
const LISTEN_DURATION_MS = 5 * 60_000; // Listen for 5 minutes
const MIN_BLOCKS_EXPECTED = 10; // Minimum blocks expected in the listen period

interface BlockRecord {
  number: bigint;
  hash: string;
  parentHash: string;
  timestamp: number;
}

interface SubstrateBlockRecord {
  number: number;
  hash: string;
  parentHash: string;
  timestamp: number;
}

interface ReorgEvent {
  type: "gap" | "chain_switch";
  previousBlock: BlockRecord;
  newBlock: BlockRecord;
  skippedBlocks: bigint[];
  timestamp: number;
}

// Per Ethereum spec (https://github.com/ethereum/go-ethereum/wiki/RPC-PUB-SUB):
// "When a chain reorganization occurs, this subscription will emit an event
// containing all new headers (blocks) for the new chain. This means that you
// may see multiple headers emitted with the same height (block number)."
//
// This test verifies that newHeads behaves according to spec during reorgs.

describeSuite({
  id: "S29",
  title: "eth_subscribe newHeads - Reorg detection and block delivery (per Ethereum spec)",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let viemClient: PublicClient;
    let wsEndpoint: string;
    let polkadotApi: ApiPromise;

    beforeAll(async () => {
      // Get the WebSocket endpoint from the context
      const httpUrl = context.viem().transport.url;
      wsEndpoint = httpUrl.replace("http://", "ws://").replace("https://", "wss://");
      log(`Using WebSocket endpoint: ${wsEndpoint}`);

      // Get the polkadotJs API for substrate-level block tracking
      polkadotApi = context.polkadotJs();
    });

    it({
      id: "C100",
      title:
        "should deliver all canonical block headers including those during reorgs (issue #3415)",
      timeout: LISTEN_DURATION_MS + 60_000, // Add buffer for setup/teardown
      test: async function () {
        const transport = webSocket(wsEndpoint);
        viemClient = createPublicClient({ transport });

        // Track blocks from eth_subscribe newHeads
        const ethBlocks: BlockRecord[] = [];
        const ethHashes = new Set<string>();

        // Track blocks from Substrate subscribeNewHeads
        const substrateBlocks: SubstrateBlockRecord[] = [];
        const substrateHashes = new Set<string>();

        // Track anomalies
        const skippedBlocks: Array<{ expected: bigint; received: bigint; gap: bigint }> = [];
        const missingParents: Array<{
          blockNumber: bigint;
          blockHash: string;
          parentHash: string;
        }> = [];
        const reorgEvents: ReorgEvent[] = [];
        const blocksNotDeliveredByEth: SubstrateBlockRecord[] = [];

        log(`Starting dual subscription test for ${LISTEN_DURATION_MS / 1000} seconds...`);
        log(`Subscribing to both eth_subscribe(newHeads) and substrate newHead...`);

        await new Promise<void>((resolve, reject) => {
          const timeoutId = setTimeout(() => {
            log("\nTest duration elapsed, cleaning up subscriptions...");
            unwatch();
            unsubscribeSubstrate?.();
            resolve();
          }, LISTEN_DURATION_MS);

          // Subscribe to eth_subscribe newHeads via viem
          const unwatch = viemClient.watchBlocks({
            onBlock: (block) => {
              const blockInfo: BlockRecord = {
                number: block.number,
                hash: block.hash,
                parentHash: block.parentHash,
                timestamp: Date.now(),
              };
              ethBlocks.push(blockInfo);

              log(
                `[ETH] Block #${block.number} hash=${block.hash.slice(0, 18)}... ` +
                  `parent=${block.parentHash.slice(0, 18)}...`
              );

              // Check for issues with the previous block
              if (ethBlocks.length > 1) {
                const prevBlock = ethBlocks[ethBlocks.length - 2];
                const expectedNumber = prevBlock.number + 1n;

                // Check 1: Gap in block numbers (explicit skip)
                if (block.number !== expectedNumber) {
                  const gap = block.number - expectedNumber;
                  skippedBlocks.push({
                    expected: expectedNumber,
                    received: block.number,
                    gap,
                  });

                  const skippedNumbers: bigint[] = [];
                  for (let i = expectedNumber; i < block.number; i++) {
                    skippedNumbers.push(i);
                  }

                  reorgEvents.push({
                    type: "gap",
                    previousBlock: prevBlock,
                    newBlock: blockInfo,
                    skippedBlocks: skippedNumbers,
                    timestamp: Date.now(),
                  });

                  log(
                    `[ETH] ♻️  REORG (GAP): Expected #${expectedNumber}, ` +
                      `received #${block.number} (${gap} blocks skipped: ${skippedNumbers.join(", ")})`
                  );
                }

                // Check 2: Chain switch - parent hash doesn't match previous block's hash
                // This is the key indicator of a reorg: we moved to a different fork
                if (block.parentHash !== prevBlock.hash) {
                  reorgEvents.push({
                    type: "chain_switch",
                    previousBlock: prevBlock,
                    newBlock: blockInfo,
                    skippedBlocks: [],
                    timestamp: Date.now(),
                  });

                  log(`[ETH] ♻️  REORG (CHAIN SWITCH) at block #${block.number}:`);
                  log(
                    `[ETH]     Previous block #${prevBlock.number}: ${prevBlock.hash.slice(0, 18)}...`
                  );
                  log(
                    `[ETH]     New block #${block.number} parent: ${block.parentHash.slice(0, 18)}...`
                  );
                  log(`[ETH]     ❌ Parent hash mismatch! Chain switched to different fork.`);
                }

                // Track missing parents for analysis (parent was never delivered to us)
                if (!ethHashes.has(block.parentHash)) {
                  missingParents.push({
                    blockNumber: block.number,
                    blockHash: block.hash,
                    parentHash: block.parentHash,
                  });
                }
              }

              ethHashes.add(block.hash);
            },
            onError: (error) => {
              log(`[ETH] Subscription error: ${error.message}`);
              clearTimeout(timeoutId);
              reject(error);
            },
          });

          // Subscribe to Substrate newHead for comparison
          let unsubscribeSubstrate: (() => void) | undefined;

          polkadotApi.rpc.chain
            .subscribeNewHeads((header) => {
              const blockInfo: SubstrateBlockRecord = {
                number: header.number.toNumber(),
                hash: header.hash.toHex(),
                parentHash: header.parentHash.toHex(),
                timestamp: Date.now(),
              };
              substrateBlocks.push(blockInfo);
              substrateHashes.add(header.hash.toHex());

              log(
                `[SUB] Block #${blockInfo.number} hash=${blockInfo.hash.slice(0, 18)}... ` +
                  `parent=${blockInfo.parentHash.slice(0, 18)}...`
              );
            })
            .then((unsub) => {
              unsubscribeSubstrate = unsub;
            })
            .catch((error) => {
              log(`[SUB] Subscription error: ${error.message}`);
              // Don't fail the test, just log
            });
        });

        // Analysis phase
        log(`\n${"=".repeat(80)}`);
        log(`SUBSCRIPTION ANALYSIS`);
        log(`${"=".repeat(80)}`);

        log(`\neth_subscribe(newHeads) summary:`);
        log(`  - Total blocks received: ${ethBlocks.length}`);
        log(`  - Unique block hashes: ${ethHashes.size}`);
        if (ethBlocks.length > 0) {
          log(`  - First block: #${ethBlocks[0].number}`);
          log(`  - Last block: #${ethBlocks[ethBlocks.length - 1].number}`);
          const range = ethBlocks[ethBlocks.length - 1].number - ethBlocks[0].number + 1n;
          log(`  - Expected range: ${range} blocks`);
          log(`  - Missing from range: ${range - BigInt(ethBlocks.length)} blocks`);
        }

        log(`\nSubstrate subscribeNewHeads summary:`);
        log(`  - Total blocks received: ${substrateBlocks.length}`);
        log(`  - Unique block hashes: ${substrateHashes.size}`);
        if (substrateBlocks.length > 0) {
          log(`  - First block: #${substrateBlocks[0].number}`);
          log(`  - Last block: #${substrateBlocks[substrateBlocks.length - 1].number}`);
        }

        // Find blocks that Substrate reported but eth_subscribe missed
        for (const subBlock of substrateBlocks) {
          if (!ethHashes.has(subBlock.hash)) {
            blocksNotDeliveredByEth.push(subBlock);
          }
        }

        if (blocksNotDeliveredByEth.length > 0) {
          log(`\n❌ BLOCKS REPORTED BY SUBSTRATE BUT NOT BY eth_subscribe:`);
          for (const block of blocksNotDeliveredByEth) {
            log(`  - Block #${block.number} (${block.hash.slice(0, 18)}...)`);
          }
        }

        // Report skipped blocks
        if (skippedBlocks.length > 0) {
          log(`\n❌ SKIPPED BLOCKS DETECTED IN eth_subscribe:`);
          for (const skip of skippedBlocks) {
            log(
              `  - Gap at block #${skip.expected}: jumped to #${skip.received} ` +
                `(${skip.gap} blocks missing)`
            );
          }
        } else {
          log(`\n✓ No gaps detected in eth_subscribe block sequence`);
        }

        // Report reorg events
        const chainSwitches = reorgEvents.filter((r) => r.type === "chain_switch");
        const gaps = reorgEvents.filter((r) => r.type === "gap");

        if (reorgEvents.length > 0) {
          log(`\n♻️  REORG EVENTS DETECTED: ${reorgEvents.length} total`);
          log(`    - Chain switches (parent mismatch): ${chainSwitches.length}`);
          log(`    - Gaps (skipped block numbers): ${gaps.length}`);

          if (chainSwitches.length > 0) {
            log(`\n  CHAIN SWITCHES (issue #3415 - blocks skipped during reorg):`);
            for (const reorg of chainSwitches) {
              log(`    Block #${reorg.newBlock.number}:`);
              log(
                `      Previous delivered: #${reorg.previousBlock.number} (${reorg.previousBlock.hash.slice(0, 18)}...)`
              );
              log(`      New block parent:   ${reorg.newBlock.parentHash.slice(0, 18)}...`);
              log(`      New block hash:     ${reorg.newBlock.hash.slice(0, 18)}...`);
              log(
                `      → The subscription jumped to a different fork without delivering intermediate blocks`
              );
            }
          }

          if (gaps.length > 0) {
            log(`\n  GAPS (explicit block number skips):`);
            for (const reorg of gaps) {
              log(`    From #${reorg.previousBlock.number} to #${reorg.newBlock.number}`);
              log(`      Skipped: ${reorg.skippedBlocks.join(", ")}`);
            }
          }
        } else {
          log(`\n✓ No reorg events detected`);
        }

        // Report missing parent blocks
        if (missingParents.length > 0) {
          log(`\n❌ MISSING PARENT BLOCKS:`);
          for (const missing of missingParents) {
            log(
              `  - Block #${missing.blockNumber}: parent ${missing.parentHash.slice(0, 18)}... ` +
                `was never delivered`
            );
          }
        } else {
          log(`\n✓ All parent hashes reference previously delivered blocks`);
        }

        // Assertions
        log(`\n${"=".repeat(80)}`);
        log(`TEST ASSERTIONS`);
        log(`${"=".repeat(80)}`);

        // Verify we received enough blocks
        expect(
          ethBlocks.length,
          `Expected at least ${MIN_BLOCKS_EXPECTED} blocks from eth_subscribe, ` +
            `received ${ethBlocks.length}`
        ).toBeGreaterThanOrEqual(MIN_BLOCKS_EXPECTED);

        // The core assertion for issue #3415: no blocks should be skipped
        expect(
          skippedBlocks,
          `eth_subscribe(newHeads) skipped blocks during potential reorgs: ${skippedBlocks
            .map((s) => `expected #${s.expected}, got #${s.received} (gap: ${s.gap})`)
            .join("; ")}`
        ).toHaveLength(0);

        // All parent blocks should have been delivered
        expect(
          missingParents,
          `Missing parent blocks detected: ${missingParents.length} blocks reference ` +
            `parents that were never delivered`
        ).toHaveLength(0);

        // No chain switches should occur (issue #3415 - this is the main bug)
        expect(
          chainSwitches,
          `Chain switch reorgs detected: ${chainSwitches.length} times the subscription ` +
            `jumped to a different fork without delivering intermediate blocks`
        ).toHaveLength(0);

        // Every block Substrate reported should also be reported by eth_subscribe
        // Note: This may have timing differences, so we log but don't fail on this
        if (blocksNotDeliveredByEth.length > 0) {
          log(
            `\n⚠️  WARNING: ${blocksNotDeliveredByEth.length} blocks were reported by ` +
              `Substrate but not by eth_subscribe. This may indicate the issue from #3415.`
          );
        }
      },
    });

    it({
      id: "C101",
      title: "per Ethereum spec: reorgs should emit same block number with different hashes",
      timeout: LISTEN_DURATION_MS + 60_000,
      test: async function () {
        // Per Ethereum spec: "When a chain reorganization occurs, this subscription
        // will emit an event containing all new headers for the new chain. This means
        // you may see multiple headers emitted with the same height (block number)."
        //
        // This test verifies that during reorgs, the subscription properly re-emits
        // block headers for the new chain, resulting in the same block number
        // appearing multiple times with different hashes.

        const transport = webSocket(wsEndpoint);
        viemClient = createPublicClient({ transport });

        const blocks: BlockRecord[] = [];
        const hashByNumber = new Map<bigint, Set<string>>();
        const chainSwitches: Array<{
          blockNumber: bigint;
          prevHash: string;
          newParentHash: string;
        }> = [];
        const duplicateNumbers: Array<{ number: bigint; hashes: string[] }> = [];

        log(`Starting Ethereum spec compliance test for ${LISTEN_DURATION_MS / 1000} seconds...`);
        log(`Per spec: reorgs should emit same block number with different hashes`);

        await new Promise<void>((resolve, reject) => {
          const timeoutId = setTimeout(() => {
            unwatch();
            resolve();
          }, LISTEN_DURATION_MS);

          const unwatch = viemClient.watchBlocks({
            onBlock: (block) => {
              const blockInfo: BlockRecord = {
                number: block.number,
                hash: block.hash,
                parentHash: block.parentHash,
                timestamp: Date.now(),
              };

              // Detect chain switches (reorgs where parent doesn't match)
              if (blocks.length > 0) {
                const prevBlock = blocks[blocks.length - 1];
                if (block.number === prevBlock.number + 1n && block.parentHash !== prevBlock.hash) {
                  chainSwitches.push({
                    blockNumber: block.number,
                    prevHash: prevBlock.hash,
                    newParentHash: block.parentHash,
                  });
                  log(
                    `[CHAIN SWITCH] Block #${block.number}: parent ${block.parentHash.slice(0, 12)}... ` +
                      `!= prev hash ${prevBlock.hash.slice(0, 12)}...`
                  );
                }
              }

              blocks.push(blockInfo);

              // Track all hashes seen for each block number
              if (!hashByNumber.has(block.number)) {
                hashByNumber.set(block.number, new Set());
              }
              const hashes = hashByNumber.get(block.number)!;
              hashes.add(block.hash);

              // Per spec, this SHOULD happen during reorgs
              if (hashes.size > 1) {
                log(
                  `[SPEC COMPLIANT] Block #${block.number} re-emitted with new hash ` +
                    `(${hashes.size} hashes seen): ${Array.from(hashes)
                      .map((h) => h.slice(0, 12) + "...")
                      .join(", ")}`
                );
              }
            },
            onError: (error) => {
              clearTimeout(timeoutId);
              reject(error);
            },
          });
        });

        // Find block numbers with multiple different hashes
        for (const [number, hashes] of hashByNumber) {
          if (hashes.size > 1) {
            duplicateNumbers.push({
              number,
              hashes: Array.from(hashes),
            });
          }
        }

        log(`\n${"=".repeat(80)}`);
        log(`ETHEREUM SPEC COMPLIANCE CHECK`);
        log(`${"=".repeat(80)}`);
        log(`\nSubscription summary:`);
        log(`  - Total blocks received: ${blocks.length}`);
        log(`  - Unique block numbers: ${hashByNumber.size}`);
        log(`  - Chain switches detected (reorgs): ${chainSwitches.length}`);
        log(`  - Block numbers with multiple hashes: ${duplicateNumbers.length}`);

        if (chainSwitches.length > 0 && duplicateNumbers.length === 0) {
          log(`\n❌ SPEC VIOLATION DETECTED:`);
          log(`   ${chainSwitches.length} chain switches occurred but NO block numbers`);
          log(`   were re-emitted with different hashes.`);
          log(`\n   Per Ethereum spec, when a reorg occurs:`);
          log(`   "this subscription will emit an event containing all new headers`);
          log(`   for the new chain. This means that you may see multiple headers`);
          log(`   emitted with the same height (block number)."`);
          log(`\n   Current behavior: subscription jumps to new fork without`);
          log(`   re-emitting headers for the new chain's blocks.`);
        }

        if (duplicateNumbers.length > 0) {
          log(`\n✓ Spec-compliant reorg handling detected:`);
          for (const dup of duplicateNumbers) {
            log(`  - Block #${dup.number}: ${dup.hashes.length} different hashes`);
            for (const hash of dup.hashes) {
              log(`      ${hash}`);
            }
          }
        }

        expect(
          blocks.length,
          `Expected at least ${MIN_BLOCKS_EXPECTED} blocks`
        ).toBeGreaterThanOrEqual(MIN_BLOCKS_EXPECTED);

        // If we detected chain switches but no duplicate block numbers,
        // the implementation is not following Ethereum spec
        if (chainSwitches.length > 0) {
          expect(
            duplicateNumbers.length,
            `Detected ${chainSwitches.length} chain switches (reorgs) but the subscription ` +
              `did NOT re-emit block headers with the same number and different hashes. ` +
              `Per Ethereum spec, reorgs should cause the same block number to be emitted ` +
              `multiple times with different hashes.`
          ).toBeGreaterThan(0);
        }

        log(`\n✓ Spec compliance check complete`);
      },
    });
  },
});
