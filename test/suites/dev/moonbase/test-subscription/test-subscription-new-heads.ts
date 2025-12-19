import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { PublicClient } from "viem";
import WebSocket from "ws";

/**
 * Redesigned test suite for eth_subscribe newHeads reorg behavior.
 *
 * Design principles (see ADR-001):
 * - Each test gets isolated WebSocket client and subscription
 * - Event-driven verification with time-windowed collection
 * - Invariant-based assertions rather than sequence checks
 * - Hash-based tracking instead of height-based waiting
 *
 * NOTE: Uses raw WebSocket instead of viem's watchBlocks to guarantee
 * message ordering. See INVESTIGATION-block-ordering.md for details.
 */

// ============================================================================
// Types
// ============================================================================

interface ReceivedBlock {
  number: bigint;
  hash: string;
  parentHash: string;
  timestamp: number;
}

interface SubscriptionHandle {
  ws: WebSocket;
  collector: BlockCollector;
  close: () => void;
}

// ============================================================================
// BlockCollector - Accumulates and organizes block notifications
// ============================================================================

class BlockCollector {
  private blocks: ReceivedBlock[] = [];
  private byHash = new Map<string, ReceivedBlock>();
  private byHeight = new Map<bigint, ReceivedBlock[]>();
  private lastBlockTime = 0;
  private logFn: (msg: string) => void;

  constructor(logFn: (msg: string) => void = console.log) {
    this.logFn = logFn;
  }

  onBlock(block: ReceivedBlock): void {
    this.blocks.push(block);
    this.byHash.set(block.hash, block);
    this.lastBlockTime = Date.now();

    if (!this.byHeight.has(block.number)) {
      this.byHeight.set(block.number, []);
    }
    this.byHeight.get(block.number)!.push(block);

    this.logFn(
      `[SUB] Block #${block.number} hash=${block.hash.slice(0, 18)}... ` +
        `parent=${block.parentHash.slice(0, 18)}...`
    );
  }

  /**
   * Wait until no new blocks arrive for the specified quiet period.
   * This indicates the subscription has "settled" after a reorg.
   */
  async waitForStability(quietPeriodMs: number, maxWaitMs = 30000): Promise<void> {
    const startTime = Date.now();

    while (Date.now() - startTime < maxWaitMs) {
      const timeSinceLastBlock = Date.now() - this.lastBlockTime;

      if (this.blocks.length > 0 && timeSinceLastBlock >= quietPeriodMs) {
        return;
      }

      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    // If we haven't received any blocks, that's also stable (just empty)
    if (this.blocks.length === 0) {
      this.logFn("[COLLECTOR] No blocks received during stability wait");
    }
  }

  /**
   * Wait until we've received at least N blocks total.
   */
  async waitForBlockCount(count: number, timeoutMs = 30000): Promise<void> {
    const startTime = Date.now();

    while (this.blocks.length < count && Date.now() - startTime < timeoutMs) {
      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    if (this.blocks.length < count) {
      throw new Error(`Timeout waiting for ${count} blocks. Only received ${this.blocks.length}`);
    }
  }

  /**
   * Wait until we've received a block with the specified hash.
   */
  async waitForHash(hash: string, timeoutMs = 10000): Promise<ReceivedBlock> {
    const startTime = Date.now();

    while (!this.byHash.has(hash) && Date.now() - startTime < timeoutMs) {
      await new Promise((resolve) => setTimeout(resolve, 50));
    }

    const block = this.byHash.get(hash);
    if (!block) {
      throw new Error(`Timeout waiting for block hash ${hash.slice(0, 18)}...`);
    }
    return block;
  }

  // Accessors
  getAll(): ReceivedBlock[] {
    return [...this.blocks];
  }

  getByHash(hash: string): ReceivedBlock | undefined {
    return this.byHash.get(hash);
  }

  hasHash(hash: string): boolean {
    return this.byHash.has(hash);
  }

  getAtHeight(height: bigint): ReceivedBlock[] {
    return this.byHeight.get(height) ?? [];
  }

  getHeightsWithForks(): Array<{ height: bigint; blocks: ReceivedBlock[] }> {
    const result: Array<{ height: bigint; blocks: ReceivedBlock[] }> = [];
    for (const [height, blocks] of this.byHeight) {
      const uniqueHashes = new Set(blocks.map((b) => b.hash));
      if (uniqueHashes.size > 1) {
        result.push({ height, blocks });
      }
    }
    return result;
  }

  /**
   * Get the last block received at each height (should be canonical after stability).
   */
  getCanonicalChain(): Map<bigint, ReceivedBlock> {
    const result = new Map<bigint, ReceivedBlock>();
    for (const [height, blocks] of this.byHeight) {
      result.set(height, blocks[blocks.length - 1]);
    }
    return result;
  }

  getCount(): number {
    return this.blocks.length;
  }

  getHeightRange(): { min: bigint; max: bigint } | null {
    if (this.byHeight.size === 0) return null;
    const heights = [...this.byHeight.keys()];
    return {
      min: heights.reduce((a, b) => (a < b ? a : b)),
      max: heights.reduce((a, b) => (a > b ? a : b)),
    };
  }

  clear(): void {
    this.blocks = [];
    this.byHash.clear();
    this.byHeight.clear();
    this.lastBlockTime = 0;
  }
}

// ============================================================================
// InvariantChecker - Verifies test invariants
// ============================================================================

class InvariantChecker {
  private collector: BlockCollector;
  private logFn: (msg: string) => void;

  constructor(collector: BlockCollector, logFn: (msg: string) => void = console.log) {
    this.collector = collector;
    this.logFn = logFn;
  }

  /**
   * Verify that a fork was visible (same height, different hashes).
   */
  checkForkVisible(height: bigint): { passed: boolean; hashes: string[] } {
    const blocks = this.collector.getAtHeight(height);
    const uniqueHashes = [...new Set(blocks.map((b) => b.hash))];

    const passed = uniqueHashes.length > 1;
    if (passed) {
      this.logFn(`✓ Fork visible at height ${height}: ${uniqueHashes.length} different blocks`);
    } else {
      this.logFn(`✗ No fork visible at height ${height}: only ${uniqueHashes.length} block(s)`);
    }

    return { passed, hashes: uniqueHashes };
  }

  /**
   * Verify that specific hashes were received.
   */
  checkHashesReceived(hashes: string[]): { passed: boolean; missing: string[] } {
    const missing = hashes.filter((h) => !this.collector.hasHash(h));
    const passed = missing.length === 0;

    if (passed) {
      this.logFn(`✓ All ${hashes.length} expected hashes received`);
    } else {
      this.logFn(`✗ Missing ${missing.length}/${hashes.length} hashes`);
      for (const h of missing) {
        this.logFn(`  - ${h.slice(0, 18)}...`);
      }
    }

    return { passed, missing };
  }

  /**
   * Verify parent chain continuity within the canonical chain.
   */
  checkParentContinuity(): { passed: boolean; breaks: Array<{ height: bigint; issue: string }> } {
    const canonical = this.collector.getCanonicalChain();
    const breaks: Array<{ height: bigint; issue: string }> = [];

    const sortedHeights = [...canonical.keys()].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));

    for (let i = 1; i < sortedHeights.length; i++) {
      const prevHeight = sortedHeights[i - 1];
      const currHeight = sortedHeights[i];

      // Only check consecutive heights
      if (currHeight !== prevHeight + 1n) continue;

      const prevBlock = canonical.get(prevHeight)!;
      const currBlock = canonical.get(currHeight)!;

      if (currBlock.parentHash !== prevBlock.hash) {
        breaks.push({
          height: currHeight,
          issue: `Parent mismatch: expected ${prevBlock.hash.slice(0, 18)}..., got ${currBlock.parentHash.slice(0, 18)}...`,
        });
      }
    }

    const passed = breaks.length === 0;
    if (passed) {
      this.logFn(`✓ Parent chain continuity verified`);
    } else {
      this.logFn(`✗ Parent chain has ${breaks.length} break(s)`);
      for (const b of breaks) {
        this.logFn(`  Height ${b.height}: ${b.issue}`);
      }
    }

    return { passed, breaks };
  }

  /**
   * Verify no block heights are skipped in the received range.
   */
  checkNoGaps(): { passed: boolean; gaps: bigint[] } {
    const range = this.collector.getHeightRange();
    if (!range) {
      this.logFn(`✓ No gaps (no blocks received)`);
      return { passed: true, gaps: [] };
    }

    const receivedHeights = new Set(this.collector.getCanonicalChain().keys());
    const gaps: bigint[] = [];

    for (let h = range.min; h <= range.max; h++) {
      if (!receivedHeights.has(h)) {
        gaps.push(h);
      }
    }

    const passed = gaps.length === 0;
    if (passed) {
      this.logFn(`✓ No gaps in height range ${range.min}-${range.max}`);
    } else {
      this.logFn(`✗ Found ${gaps.length} gap(s): ${gaps.join(", ")}`);
    }

    return { passed, gaps };
  }

  /**
   * Verify we received the RPC canonical block at each height.
   * During reorgs, we might receive multiple blocks at a height - that's OK
   * as long as the canonical block was among them.
   */
  async checkReceivedCanonicalBlocks(
    viem: PublicClient
  ): Promise<{ passed: boolean; missing: Array<{ height: bigint; rpcHash: string }> }> {
    const range = this.collector.getHeightRange();
    if (!range) {
      this.logFn(`✓ No blocks to check`);
      return { passed: true, missing: [] };
    }

    const missing: Array<{ height: bigint; rpcHash: string }> = [];

    for (let height = range.min; height <= range.max; height++) {
      try {
        const rpcBlock = await viem.getBlock({ blockNumber: height });
        const blocksAtHeight = this.collector.getAtHeight(height);
        const receivedCanonical = blocksAtHeight.some((b) => b.hash === rpcBlock.hash);

        if (!receivedCanonical) {
          missing.push({ height, rpcHash: rpcBlock.hash });
        }
      } catch {
        // Block might not exist, skip
      }
    }

    const passed = missing.length === 0;
    if (passed) {
      this.logFn(`✓ All canonical blocks were received`);
    } else {
      this.logFn(`✗ Missing ${missing.length} canonical block(s)`);
      for (const m of missing) {
        this.logFn(
          `  Height ${m.height}: canonical=${m.rpcHash.slice(0, 18)}... not in subscription`
        );
      }
    }

    return { passed, missing };
  }
}

// ============================================================================
// Subscription Factory - Uses raw WebSocket for guaranteed ordering
// ============================================================================

/**
 * Create a raw WebSocket subscription for newHeads.
 * This guarantees message ordering unlike viem's watchBlocks.
 */
async function createSubscription(
  wsEndpoint: string,
  logFn: (msg: string) => void
): Promise<SubscriptionHandle> {
  const collector = new BlockCollector(logFn);
  const ws = new WebSocket(wsEndpoint);

  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error("WebSocket connection timeout")), 10000);

    ws.on("open", () => {
      clearTimeout(timeout);
      // Subscribe to newHeads
      ws.send(
        JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "eth_subscribe",
          params: ["newHeads"],
        })
      );
    });

    ws.on("message", (data: Buffer) => {
      const msg = JSON.parse(data.toString());

      if (msg.id === 1) {
        // Subscription confirmed
        resolve();
      } else if (msg.method === "eth_subscription") {
        const block = msg.params?.result;
        if (block) {
          collector.onBlock({
            number: BigInt(block.number),
            hash: block.hash,
            parentHash: block.parentHash,
            timestamp: Date.now(),
          });
        }
      }
    });

    ws.on("error", (err) => {
      clearTimeout(timeout);
      reject(err);
    });
  });

  return {
    ws,
    collector,
    close: () => ws.close(),
  };
}

/**
 * Helper to ensure subscription is receiving blocks.
 * Creates blocks until at least one is received.
 */
async function warmupSubscription(
  sub: SubscriptionHandle,
  createBlockFn: () => Promise<unknown>,
  options: { maxAttempts?: number; delayMs?: number } = {}
): Promise<void> {
  const maxAttempts = options.maxAttempts ?? 10;
  const delayMs = options.delayMs ?? 300;

  const initialCount = sub.collector.getCount();

  for (let i = 0; i < maxAttempts; i++) {
    await createBlockFn();
    await new Promise((resolve) => setTimeout(resolve, delayMs));

    if (sub.collector.getCount() > initialCount) {
      return;
    }
  }

  throw new Error(`Subscription not receiving blocks after ${maxAttempts} warmup attempts`);
}

// ============================================================================
// Test Suite
// ============================================================================

describeSuite({
  id: "D023506",
  title: "Subscription - newHeads reorg behavior v2 (Ethereum spec compliance)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let wsEndpoint: string;

    beforeAll(async () => {
      wsEndpoint = context.viem().transport.url.replace("http", "ws");
    });

    // Helper to get Ethereum block hash
    const getEthHash = async (blockNum: bigint): Promise<string> => {
      const block = await context.viem().getBlock({ blockNumber: blockNum });
      return block.hash;
    };

    it({
      id: "T01",
      title: "should detect fork during reorg (both fork blocks visible)",
      test: async function () {
        log("\n=== T01: Fork Detection Test ===");

        // Create subscription
        const sub = await createSubscription(wsEndpoint, log);

        try {
          // Warmup: create blocks until subscription receives one
          await warmupSubscription(sub, () => context.createBlock([], {}));

          // Get current state
          const initialBlockNum = await context.viem().getBlockNumber();

          // Create base block
          const block1 = await context.createBlock([], {});
          const block1Num = initialBlockNum + 1n;
          const block1EthHash = await getEthHash(block1Num);
          log(`Base block #${block1Num}: ${block1EthHash.slice(0, 18)}...`);

          await sub.collector.waitForHash(block1EthHash);

          // Create Fork A (block 2a)
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          const block2Num = block1Num + 1n;
          const block2aEthHash = await getEthHash(block2Num);
          log(`Fork A block #${block2Num}: ${block2aEthHash.slice(0, 18)}...`);

          await sub.collector.waitForHash(block2aEthHash);

          // Create Fork B (block 2b) - triggers reorg detection
          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          log(`Fork B block #${block2Num} (substrate): ${block2b.block.hash.slice(0, 18)}...`);

          // Extend Fork B to make it canonical
          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          const block3Num = block2Num + 1n;

          // Wait for stability
          await sub.collector.waitForStability(2000);

          // Get canonical hashes after reorg
          const block2bEthHash = await getEthHash(block2Num);
          const block3bEthHash = await getEthHash(block3Num);
          log(`Fork B canonical #${block2Num}: ${block2bEthHash.slice(0, 18)}...`);
          log(`Fork B canonical #${block3Num}: ${block3bEthHash.slice(0, 18)}...`);

          // Verify invariants
          const checker = new InvariantChecker(sub.collector, log);

          log("\n=== Invariant Checks ===");

          // Check fork was visible at height 2
          const forkCheck = checker.checkForkVisible(block2Num);
          expect(forkCheck.passed, "Fork should be visible at height 2").toBe(true);

          // Check both fork blocks were received
          const hashCheck = checker.checkHashesReceived([block2aEthHash, block2bEthHash]);
          expect(hashCheck.passed, "Both fork blocks should be received").toBe(true);

          // Check parent continuity
          const parentCheck = checker.checkParentContinuity();
          expect(parentCheck.passed, "Parent chain should be continuous").toBe(true);

          log("\n=== T01 Complete ===");
        } finally {
          sub.close();
        }
      },
    });

    it({
      id: "T02",
      title: "should maintain canonical chain consistency with RPC after reorg",
      test: async function () {
        log("\n=== T02: Canonical Consistency Test ===");

        const sub = await createSubscription(wsEndpoint, log);

        try {
          // Warmup
          await warmupSubscription(sub, () => context.createBlock([], {}));

          // Create a simple reorg scenario - use stability waits instead of specific hash waits
          const block1 = await context.createBlock([], {});
          await sub.collector.waitForStability(500);

          // Fork A
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          // Fork B (competing)
          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });

          // Extend Fork B
          await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });

          // Wait for stability
          await sub.collector.waitForStability(2000);

          // Verify invariants
          const checker = new InvariantChecker(sub.collector, log);

          log("\n=== Invariant Checks ===");

          // We should have received all canonical blocks
          const canonicalCheck = await checker.checkReceivedCanonicalBlocks(context.viem());
          expect(canonicalCheck.passed, "Should receive all canonical blocks").toBe(true);

          // No gaps
          const gapCheck = checker.checkNoGaps();
          expect(gapCheck.passed, "No gaps in block heights").toBe(true);

          log("\n=== T02 Complete ===");
        } finally {
          sub.close();
        }
      },
    });

    it({
      id: "T03",
      title: "should handle deep reorg (multiple block rollback)",
      test: async function () {
        log("\n=== T03: Deep Reorg Test ===");

        const sub = await createSubscription(wsEndpoint, log);

        try {
          // Warmup
          await warmupSubscription(sub, () => context.createBlock([], {}));

          // Base block
          const block1 = await context.createBlock([], {});
          await sub.collector.waitForStability(500);

          // Create BOTH fork points at height 2 first (avoids long-range attack protection)
          log("\n--- Creating fork points ---");
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          // Extend Fork A to height 4
          log("\n--- Extending Fork A ---");
          const block3a = await context.createBlock([], {
            parentHash: block2a.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const _block4a = await context.createBlock([], {
            parentHash: block3a.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          // Extend Fork B to height 5 (triggers deep reorg)
          log("\n--- Extending Fork B (triggers deep reorg) ---");
          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const block4b = await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const _block5b = await context.createBlock([], {
            parentHash: block4b.block.hash,
            finalize: false,
          });

          // Wait for stability
          await sub.collector.waitForStability(2000);

          // Verify invariants
          const checker = new InvariantChecker(sub.collector, log);

          log("\n=== Invariant Checks ===");

          // Check if forks were visible (informational - Ethereum spec says they should be)
          const forks = sub.collector.getHeightsWithForks();
          log(`Heights with forks: ${forks.map((f) => f.height).join(", ") || "(none)"}`);
          if (forks.length >= 1) {
            log("✓ Fork headers were re-emitted during reorg (Ethereum spec compliant)");
          } else {
            log("⚠ No fork headers re-emitted (Ethereum spec expects re-emission)");
          }

          // Key invariant: We should have received all canonical blocks
          const canonicalCheck = await checker.checkReceivedCanonicalBlocks(context.viem());
          expect(
            canonicalCheck.passed,
            "Should receive all canonical blocks after deep reorg"
          ).toBe(true);

          // No gaps in received blocks
          const gapCheck = checker.checkNoGaps();
          expect(gapCheck.passed, "No gaps in block heights").toBe(true);

          log("\n=== T03 Complete ===");
        } finally {
          sub.close();
        }
      },
    });

    it({
      id: "T04",
      title: "new subscription after reorg should only see canonical chain",
      test: async function () {
        log("\n=== T04: New Subscription Post-Reorg Test ===");

        // Create reorg scenario WITHOUT subscription
        const block1 = await context.createBlock([], {});

        const block2a = await context.createBlock([], {
          parentHash: block1.block.hash,
          finalize: false,
        });
        // Get block2a's hash while it's still canonical (before reorg)
        const block2aHash = await getEthHash(await context.viem().getBlockNumber());
        log(`Pre-sub Fork A: ${block2aHash.slice(0, 18)}...`);

        const block2b = await context.createBlock([], {
          parentHash: block1.block.hash,
          finalize: false,
        });

        // Extend Fork B to make it canonical
        const block3b = await context.createBlock([], {
          parentHash: block2b.block.hash,
          finalize: false,
        });

        // NOW create subscription
        log("\n--- Creating subscription after reorg ---");
        const sub = await createSubscription(wsEndpoint, log);

        try {
          // Warmup with blocks extending the canonical chain
          let lastBlock = block3b;
          await warmupSubscription(sub, async () => {
            lastBlock = await context.createBlock([], {
              parentHash: lastBlock.block.hash,
              finalize: false,
            });
          });

          // Create more blocks
          const block4 = await context.createBlock([], {
            parentHash: lastBlock.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const _block5 = await context.createBlock([], {
            parentHash: block4.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(1000);

          // Verify: should NOT have received orphaned block2a
          log("\n=== Invariant Checks ===");

          const receivedOrphan = sub.collector.hasHash(block2aHash);
          log(
            `Orphaned block ${block2aHash.slice(0, 18)}...: ${receivedOrphan ? "✗ RECEIVED (bad)" : "✓ NOT received (good)"}`
          );
          expect(receivedOrphan, "Should not receive orphaned fork block").toBe(false);

          // Parent continuity
          const checker = new InvariantChecker(sub.collector, log);
          const parentCheck = checker.checkParentContinuity();
          expect(parentCheck.passed, "Parent chain should be continuous").toBe(true);

          log("\n=== T04 Complete ===");
        } finally {
          sub.close();
        }
      },
    });

    it({
      id: "T05",
      title: "should not skip block numbers during reorg",
      test: async function () {
        log("\n=== T05: No Gaps Test ===");

        const sub = await createSubscription(wsEndpoint, log);

        try {
          // Warmup
          await warmupSubscription(sub, () => context.createBlock([], {}));

          // Create reorg scenario
          const block1 = await context.createBlock([], {});
          await sub.collector.waitForStability(500);

          // Fork points
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          // Extend both forks
          await context.createBlock([], {
            parentHash: block2a.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          await sub.collector.waitForStability(500);

          await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false,
          });

          await sub.collector.waitForStability(2000);

          // Verify no gaps
          const checker = new InvariantChecker(sub.collector, log);

          log("\n=== Invariant Checks ===");

          const gapCheck = checker.checkNoGaps();
          expect(gapCheck.passed, "Should not skip block numbers").toBe(true);

          const range = sub.collector.getHeightRange();
          if (range) {
            log(`Block range: ${range.min} - ${range.max}`);
          }

          log("\n=== T05 Complete ===");
        } finally {
          sub.close();
        }
      },
    });

    it({
      id: "T06",
      title: "should reject long-range fork attempts (Substrate protection)",
      test: async function () {
        log("\n=== T06: Long-Range Attack Protection Test ===");

        // This test finalizes blocks, affecting subsequent tests
        // Create and finalize a chain
        const block1 = await context.createBlock([], {});

        const block2 = await context.createBlock([], {
          parentHash: block1.block.hash,
          finalize: false,
        });

        const block3 = await context.createBlock([], {
          parentHash: block2.block.hash,
          finalize: false,
        });

        // Finalize to establish canonical chain
        await context.createBlock([], {
          parentHash: block3.block.hash,
          finalize: true,
        });

        log("Created and finalized chain: block1 → block2 → block3 → block4");

        // Attempt to fork from block1 (now behind finalization)
        log("\n--- Attempting long-range fork ---");

        let caughtError: Error | null = null;
        try {
          await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
        } catch (error) {
          caughtError = error as Error;
          log(`✓ Caught expected error: ${caughtError.message.slice(0, 60)}...`);
        }

        // Verify
        log("\n=== Invariant Checks ===");

        expect(caughtError, "Should throw error for long-range fork").not.toBeNull();
        expect(
          caughtError?.message.includes("long-range attack"),
          "Error should mention long-range attack"
        ).toBe(true);

        log("✓ Substrate correctly rejected long-range fork attempt");
        log("\n=== T06 Complete ===");
      },
    });
  },
});
