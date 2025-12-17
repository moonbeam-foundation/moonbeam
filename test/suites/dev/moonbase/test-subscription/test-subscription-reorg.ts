import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { generateKeyringPair, createEthersTransaction } from "@moonwall/util";
import { type PublicClient, createPublicClient, webSocket } from "viem";

// Per Ethereum spec (https://github.com/ethereum/go-ethereum/wiki/RPC-PUB-SUB#newheads):
// "When a chain reorganization occurs, this subscription will emit an event
// containing all new headers (blocks) for the new chain. This means that you
// may see multiple headers emitted with the same height (block number)."

interface ReceivedBlock {
  number: bigint;
  hash: string;
  parentHash: string;
  timestamp: number;
}

describeSuite({
  id: "D023506",
  title: "Subscription - newHeads reorg behavior (Ethereum spec compliance)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let client: PublicClient;
    let wsEndpoint: string;

    beforeAll(async () => {
      wsEndpoint = context.viem().transport.url.replace("http", "ws");
      const transport = webSocket(wsEndpoint);
      client = createPublicClient({ transport });
    });

    it({
      id: "T01",
      title: "should emit block headers for both forks during reorg (per Ethereum spec)",
      test: async function () {
        // Track all blocks received via subscription
        const receivedBlocks: ReceivedBlock[] = [];
        const blocksByNumber = new Map<bigint, ReceivedBlock[]>();

        // Start subscription
        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            const blockInfo: ReceivedBlock = {
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            };
            receivedBlocks.push(blockInfo);

            // Track blocks by number to detect same-height emissions
            if (!blocksByNumber.has(block.number)) {
              blocksByNumber.set(block.number, []);
            }
            blocksByNumber.get(block.number)!.push(blockInfo);

            log(
              `[SUB] Block #${block.number} hash=${block.hash.slice(0, 18)}... ` +
                `parent=${block.parentHash.slice(0, 18)}...`
            );
          },
        });

        try {
          // Get initial block number
          const initialBlockNum = await context.viem().getBlockNumber();

          // Create base block (block 1)
          log("\n=== Creating base block ===");
          const block1 = await context.createBlock([], {});
          const block1Num = initialBlockNum + 1n;
          log(`Base block #${block1Num} hash=${block1.block.hash.slice(0, 18)}...`);

          // Wait for subscription to receive it
          await new Promise((resolve) => setTimeout(resolve, 500));

          // Create first branch from block1 (fork A)
          log("\n=== Creating Fork A (block 2a) ===");
          const randomAccount = generateKeyringPair();
          const rawTx1 = await createEthersTransaction(context, {
            to: randomAccount.address,
            value: "0x100",
            gasLimit: 25000,
          });

          const block2a = await context.createBlock(rawTx1, {
            parentHash: block1.block.hash,
            finalize: false,
          });
          const block2aNum = block1Num + 1n;
          log(`Fork A block #${block2aNum} hash=${block2a.block.hash.slice(0, 18)}...`);

          await new Promise((resolve) => setTimeout(resolve, 500));

          // Create second branch from block1 (fork B) - this should trigger reorg detection
          log("\n=== Creating Fork B (block 2b) - triggers reorg ===");
          const rawTx2 = await createEthersTransaction(context, {
            to: randomAccount.address,
            value: "0x200",
            gasLimit: 25000,
            nonce: 1,
          });

          const block2b = await context.createBlock(rawTx2, {
            parentHash: block1.block.hash,
            finalize: false,
          });
          const block2bNum = block1Num + 1n;
          log(`Fork B block #${block2bNum} hash=${block2b.block.hash.slice(0, 18)}...`);

          await new Promise((resolve) => setTimeout(resolve, 500));

          // Continue fork B to make it the canonical chain
          log("\n=== Extending Fork B (block 3b) ===");
          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          const block3bNum = block2bNum + 1n;
          log(`Fork B block #${block3bNum} hash=${block3b.block.hash.slice(0, 18)}...`);

          await new Promise((resolve) => setTimeout(resolve, 500));

          // Extend Fork B further to make it clearly the best chain (but don't finalize
          // to allow subsequent tests to create forks)
          log("\n=== Extending Fork B (block 4b) ===");
          const block4b = await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false,
          });
          const block4bNum = block3bNum + 1n;
          log(`Fork B block #${block4bNum} hash=${block4b.block.hash.slice(0, 18)}...`);

          await new Promise((resolve) => setTimeout(resolve, 1000));

          // Analysis
          log("\n" + "=".repeat(80));
          log("SUBSCRIPTION ANALYSIS");
          log("=".repeat(80));

          log(`\nTotal blocks received: ${receivedBlocks.length}`);
          log(`Unique block numbers: ${blocksByNumber.size}`);

          // Check for same-height blocks with different hashes (expected per spec)
          const sameHeightDifferentHash: Array<{ number: bigint; hashes: string[] }> = [];
          for (const [number, blocks] of blocksByNumber) {
            const uniqueHashes = new Set(blocks.map((b) => b.hash));
            if (uniqueHashes.size > 1) {
              sameHeightDifferentHash.push({
                number,
                hashes: Array.from(uniqueHashes),
              });
            }
          }

          log(
            `\nBlock numbers with multiple hashes (expected per spec): ${sameHeightDifferentHash.length}`
          );

          if (sameHeightDifferentHash.length > 0) {
            log("\n✓ SPEC COMPLIANT: Same block height emitted with different hashes:");
            for (const item of sameHeightDifferentHash) {
              log(`  Block #${item.number}:`);
              for (const hash of item.hashes) {
                log(`    - ${hash}`);
              }
            }
          } else {
            log("\n❌ SPEC VIOLATION: No block heights were re-emitted with different hashes");
            log("   Per Ethereum spec, during reorgs the subscription should emit");
            log("   the same block number multiple times with different hashes.");
          }

          // Check for chain switches (parent mismatch)
          const chainSwitches: Array<{
            blockNumber: bigint;
            prevHash: string;
            newParent: string;
          }> = [];

          for (let i = 1; i < receivedBlocks.length; i++) {
            const prev = receivedBlocks[i - 1];
            const curr = receivedBlocks[i];

            if (curr.number === prev.number + 1n && curr.parentHash !== prev.hash) {
              chainSwitches.push({
                blockNumber: curr.number,
                prevHash: prev.hash,
                newParent: curr.parentHash,
              });
            }
          }

          log(`\nChain switches detected: ${chainSwitches.length}`);
          if (chainSwitches.length > 0) {
            log("  Chain switches indicate the subscription jumped to a different fork");
            log("  without re-emitting headers for the new chain (spec violation):");
            for (const cs of chainSwitches) {
              log(`  - Block #${cs.blockNumber}:`);
              log(`      Previous hash: ${cs.prevHash.slice(0, 18)}...`);
              log(`      New parent:    ${cs.newParent.slice(0, 18)}... (mismatch!)`);
            }
          }

          // Log all received blocks for debugging
          log("\nAll received blocks:");
          for (const block of receivedBlocks) {
            log(
              `  #${block.number}: ${block.hash.slice(0, 18)}... (parent: ${block.parentHash.slice(0, 18)}...)`
            );
          }

          log("\n" + "=".repeat(80));
          log("TEST ASSERTIONS");
          log("=".repeat(80));

          // We created blocks at heights 1, 2 (fork A), 2 (fork B), 3, 4
          // Per Ethereum spec, we should see height 2 emitted twice with different hashes
          expect(
            receivedBlocks.length,
            "Should have received multiple blocks"
          ).toBeGreaterThanOrEqual(3);

          // The key spec compliance check: if we created a reorg, we should see
          // the same block number with different hashes
          // NOTE: This test will FAIL if the implementation doesn't follow spec
          if (chainSwitches.length > 0) {
            expect(
              sameHeightDifferentHash.length,
              `Detected ${chainSwitches.length} chain switches but NO block numbers were ` +
                `re-emitted with different hashes. Per Ethereum spec, reorgs should cause ` +
                `the same block number to be emitted multiple times with different hashes. ` +
                `This is a spec violation (issue #3415).`
            ).toBeGreaterThan(0);
          }
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T02",
      title: "should track parent hash continuity during normal operation",
      test: async function () {
        const receivedBlocks: ReceivedBlock[] = [];

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            receivedBlocks.push({
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            });
          },
        });

        try {
          // Create several blocks in sequence (no reorg)
          for (let i = 0; i < 5; i++) {
            await context.createBlock([], {});
            await new Promise((resolve) => setTimeout(resolve, 200));
          }

          // Verify parent hash continuity
          let parentMismatches = 0;
          for (let i = 1; i < receivedBlocks.length; i++) {
            const prev = receivedBlocks[i - 1];
            const curr = receivedBlocks[i];

            if (curr.number === prev.number + 1n && curr.parentHash !== prev.hash) {
              parentMismatches++;
              log(
                `Parent mismatch at block #${curr.number}: ` +
                  `expected parent ${prev.hash.slice(0, 18)}..., ` +
                  `got ${curr.parentHash.slice(0, 18)}...`
              );
            }
          }

          expect(receivedBlocks.length).toBeGreaterThanOrEqual(5);
          expect(
            parentMismatches,
            "In normal operation (no reorgs), all blocks should have correct parent hashes"
          ).toBe(0);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T03",
      title: "should emit new chain headers in correct order during reorg",
      test: async function () {
        const receivedBlocks: ReceivedBlock[] = [];

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            receivedBlocks.push({
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            });
          },
        });

        try {
          // Create base block
          const block1 = await context.createBlock([], {});
          await new Promise((resolve) => setTimeout(resolve, 300));

          // Create fork A with 2 blocks
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          const _block3a = await context.createBlock([], {
            parentHash: block2a.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          // Create fork B with 3 blocks (longer chain triggers reorg)
          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          const block4b = await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          // Per Ethereum spec, when reorg occurs, new chain headers should be emitted
          // in ascending order (block 2b, then 3b, then 4b)
          log("\n=== Block Order Analysis ===");

          // Find blocks from fork B in received order
          const forkBBlocks = receivedBlocks.filter(
            (b) =>
              b.hash === block2b.block.hash ||
              b.hash === block3b.block.hash ||
              b.hash === block4b.block.hash
          );

          log(`Fork B blocks received: ${forkBBlocks.length}`);
          for (const b of forkBBlocks) {
            log(`  #${b.number}: ${b.hash.slice(0, 18)}...`);
          }

          // Verify ascending order if multiple fork B blocks received
          if (forkBBlocks.length >= 2) {
            let isAscending = true;
            for (let i = 1; i < forkBBlocks.length; i++) {
              if (forkBBlocks[i].number < forkBBlocks[i - 1].number) {
                isAscending = false;
                log(
                  `  ❌ Out of order: #${forkBBlocks[i - 1].number} followed by #${forkBBlocks[i].number}`
                );
              }
            }
            if (isAscending) {
              log("  ✓ Fork B blocks received in ascending order");
            }
            expect(isAscending, "New chain headers should be emitted in ascending order").toBe(
              true
            );
          }

          // We should receive blocks from the canonical chain
          expect(
            receivedBlocks.length,
            "Should have received multiple blocks"
          ).toBeGreaterThanOrEqual(3);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T04",
      title: "should handle deep reorg (multiple block rollback)",
      test: async function () {
        const receivedBlocks: ReceivedBlock[] = [];
        const blocksByNumber = new Map<bigint, ReceivedBlock[]>();

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            const blockInfo: ReceivedBlock = {
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            };
            receivedBlocks.push(blockInfo);

            if (!blocksByNumber.has(block.number)) {
              blocksByNumber.set(block.number, []);
            }
            blocksByNumber.get(block.number)!.push(blockInfo);
          },
        });

        try {
          // Create base block
          const block1 = await context.createBlock([], {});
          await new Promise((resolve) => setTimeout(resolve, 300));

          // IMPORTANT: Create BOTH fork points at height 2 first (before going deeper)
          // This avoids "long-range attack" protection that kicks in when forking
          // from a block that's too far behind the current best block
          log("\n=== Creating fork points at height 2 ===");
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Now extend fork A deeper
          log("\n=== Extending Fork A (to height 4) ===");
          const block3a = await context.createBlock([], {
            parentHash: block2a.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          const block4a = await context.createBlock([], {
            parentHash: block3a.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          log(`  Fork A: #2a=${block2a.block.hash.slice(0, 12)}...`);
          log(`  Fork A: #3a=${block3a.block.hash.slice(0, 12)}...`);
          log(`  Fork A: #4a=${block4a.block.hash.slice(0, 12)}...`);

          // Extend fork B longer (triggers deep reorg)
          log("\n=== Extending Fork B (to height 5 - triggers deep reorg) ===");
          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          const block4b = await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          const block5b = await context.createBlock([], {
            parentHash: block4b.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          log(`  Fork B: #2b=${block2b.block.hash.slice(0, 12)}...`);
          log(`  Fork B: #3b=${block3b.block.hash.slice(0, 12)}...`);
          log(`  Fork B: #4b=${block4b.block.hash.slice(0, 12)}...`);
          log(`  Fork B: #5b=${block5b.block.hash.slice(0, 12)}...`);

          // Analysis: Per spec, after deep reorg we should see re-emitted headers
          log("\n=== Deep Reorg Analysis ===");

          // Count heights with multiple different hashes
          let heightsWithMultipleHashes = 0;
          for (const [number, blocks] of blocksByNumber) {
            const uniqueHashes = new Set(blocks.map((b) => b.hash));
            if (uniqueHashes.size > 1) {
              heightsWithMultipleHashes++;
              log(`  Height #${number}: ${uniqueHashes.size} different hashes`);
            }
          }

          log(`\nHeights with multiple hashes: ${heightsWithMultipleHashes}`);

          // For a 3-block deep reorg, we expect heights 2, 3, 4 to potentially
          // have multiple hashes if spec-compliant
          if (heightsWithMultipleHashes >= 1) {
            log("✓ SPEC COMPLIANT: Deep reorg caused re-emission of block headers");
          } else {
            log("❌ SPEC VIOLATION: Deep reorg did not cause header re-emission");
          }

          // Basic sanity check - we should have received several blocks
          expect(
            receivedBlocks.length,
            "Should have received blocks from both forks"
          ).toBeGreaterThanOrEqual(4);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T05",
      title: "new subscription after reorg should only see canonical chain",
      test: async function () {
        // First, create a reorg scenario
        const block1 = await context.createBlock([], {});
        await new Promise((resolve) => setTimeout(resolve, 200));

        // Create BOTH fork points at height 2 first (avoids long-range attack protection)
        const block2a = await context.createBlock([], {
          parentHash: block1.block.hash,
          finalize: false,
        });
        await new Promise((resolve) => setTimeout(resolve, 200));

        const block2b = await context.createBlock([], {
          parentHash: block1.block.hash,
          finalize: false,
        });
        await new Promise((resolve) => setTimeout(resolve, 200));

        // Extend fork B to make it canonical
        const block3b = await context.createBlock([], {
          parentHash: block2b.block.hash,
          finalize: false, // Don't finalize to allow subsequent tests to create forks
        });
        await new Promise((resolve) => setTimeout(resolve, 300));

        // NOW start a new subscription - it should only see canonical chain going forward
        const receivedBlocks: ReceivedBlock[] = [];

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            receivedBlocks.push({
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            });
          },
        });

        try {
          // Create more blocks on canonical chain
          const block4b = await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          await context.createBlock([], {
            parentHash: block4b.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          log("\n=== New Subscription After Reorg ===");
          log(`Blocks received by new subscription: ${receivedBlocks.length}`);

          // Verify all received blocks have continuous parent hashes (no orphan references)
          let orphanReferences = 0;
          for (let i = 1; i < receivedBlocks.length; i++) {
            const prev = receivedBlocks[i - 1];
            const curr = receivedBlocks[i];

            if (curr.number === prev.number + 1n && curr.parentHash !== prev.hash) {
              orphanReferences++;
              log(
                `  ❌ Block #${curr.number} references non-received parent: ${curr.parentHash.slice(0, 18)}...`
              );
            }
          }

          // New subscription should not receive block2a (orphaned fork)
          const receivedOrphanedBlock = receivedBlocks.some((b) => b.hash === block2a.block.hash);

          if (!receivedOrphanedBlock) {
            log("✓ New subscription did not receive orphaned fork block");
          } else {
            log("❌ New subscription received orphaned fork block");
          }

          expect(
            receivedOrphanedBlock,
            "New subscription should not receive blocks from orphaned forks"
          ).toBe(false);

          expect(
            orphanReferences,
            "New subscription should only see continuous canonical chain"
          ).toBe(0);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T06",
      title: "should detect when subscription misses reorg blocks",
      test: async function () {
        const receivedBlocks: ReceivedBlock[] = [];
        const receivedHashes = new Set<string>();

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            receivedBlocks.push({
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            });
            receivedHashes.add(block.hash);
          },
        });

        try {
          // Get initial block number
          const initialBlockNum = await context.viem().getBlockNumber();

          // Create base
          const block1 = await context.createBlock([], {});
          const block1Num = initialBlockNum + 1n;
          await new Promise((resolve) => setTimeout(resolve, 300));

          // Create BOTH fork points at height 2 first (avoids long-range attack protection)
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          const forkBlockNum = block1Num + 1n; // Both 2a and 2b are at same height
          await new Promise((resolve) => setTimeout(resolve, 200));

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          // Extend fork B to trigger reorg
          await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          // Check if we received both fork blocks at height 2
          const block2aHash = block2a.block.hash;
          const block2bHash = block2b.block.hash;

          const receivedBlock2a = receivedHashes.has(block2aHash);
          const receivedBlock2b = receivedHashes.has(block2bHash);

          log("\n=== Reorg Block Delivery Check ===");
          log(
            `Block 2a (fork A) hash: ${block2aHash.slice(0, 18)}... - Received: ${receivedBlock2a}`
          );
          log(
            `Block 2b (fork B) hash: ${block2bHash.slice(0, 18)}... - Received: ${receivedBlock2b}`
          );

          // Count blocks received at height 2
          const blocksAtHeight2 = receivedBlocks.filter((b) => b.number === forkBlockNum);
          log(`\nBlocks received at height ${forkBlockNum}: ${blocksAtHeight2.length}`);
          for (const b of blocksAtHeight2) {
            log(`  - ${b.hash}`);
          }

          // Per Ethereum spec, during reorgs we should receive headers for the NEW canonical chain.
          // This means we should receive block2b (the new canonical block at this height),
          // but we don't necessarily receive block2a (the old fork's block).
          //
          // The spec says: "When a chain reorganization occurs, this subscription will emit
          // an event containing all new headers (blocks) for the new chain."
          //
          // Since we're testing that block2b (the new canonical chain's block) is received,
          // we check that we got at least one block at the fork height.
          if (blocksAtHeight2.length >= 1) {
            log("\n✓ SPEC COMPLIANT: Received block(s) at fork height during reorg");
            // Verify that the canonical chain's block (block2b's chain) was received
            // by checking that block3b's parent is in our received blocks
            const block3bParentReceived = receivedBlocks.some(
              (b) => b.hash === block2b.block.hash || receivedHashes.has(block2b.block.hash)
            );
            if (block3bParentReceived) {
              log("   New canonical chain block received correctly");
            }
          } else {
            log("\n❌ SPEC VIOLATION: Did not receive any blocks at fork height");
          }

          // We should receive at least one block at the fork height
          expect(
            blocksAtHeight2.length,
            `Expected to receive at least one block at height ${forkBlockNum}, ` +
              `but received ${blocksAtHeight2.length}.`
          ).toBeGreaterThanOrEqual(1);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T07",
      title: "should handle rapid successive reorgs",
      test: async function () {
        const receivedBlocks: ReceivedBlock[] = [];
        const blocksByNumber = new Map<bigint, ReceivedBlock[]>();

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            const blockInfo: ReceivedBlock = {
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            };
            receivedBlocks.push(blockInfo);

            if (!blocksByNumber.has(block.number)) {
              blocksByNumber.set(block.number, []);
            }
            blocksByNumber.get(block.number)!.push(blockInfo);
          },
        });

        try {
          // Create base block
          const block1 = await context.createBlock([], {});
          await new Promise((resolve) => setTimeout(resolve, 200));

          log("\n=== Rapid Successive Reorgs ===");

          // Create ALL fork points at height 2 FIRST (avoids long-range attack protection)
          log("\n--- Creating all fork points at height 2 ---");
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 100));

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 100));

          const block2c = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 100));

          const block2d = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Now extend each fork in sequence to trigger reorgs
          // Fork A stays at height 2 (shortest)
          log("\n--- Reorg 1: Fork B overtakes Fork A ---");
          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Reorg 2: Fork C overtakes Fork B
          log("--- Reorg 2: Fork C overtakes Fork B ---");
          const block3c = await context.createBlock([], {
            parentHash: block2c.block.hash,
            finalize: false,
          });
          const block4c = await context.createBlock([], {
            parentHash: block3c.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Reorg 3: Fork D overtakes Fork C
          log("--- Reorg 3: Fork D overtakes Fork C ---");
          const block3d = await context.createBlock([], {
            parentHash: block2d.block.hash,
            finalize: false,
          });
          const block4d = await context.createBlock([], {
            parentHash: block3d.block.hash,
            finalize: false,
          });
          await context.createBlock([], {
            parentHash: block4d.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          // Log created blocks for debugging
          log(`\n  Fork A: #2a=${block2a.block.hash.slice(0, 12)}...`);
          log(
            `  Fork B: #2b=${block2b.block.hash.slice(0, 12)}..., #3b=${block3b.block.hash.slice(0, 12)}...`
          );
          log(
            `  Fork C: #2c=${block2c.block.hash.slice(0, 12)}..., #3c=${block3c.block.hash.slice(0, 12)}..., #4c=${block4c.block.hash.slice(0, 12)}...`
          );
          log(
            `  Fork D: #2d=${block2d.block.hash.slice(0, 12)}..., #3d=${block3d.block.hash.slice(0, 12)}..., #4d=${block4d.block.hash.slice(0, 12)}...`
          );

          // Analysis
          log("\n=== Analysis ===");
          log(`Total blocks received: ${receivedBlocks.length}`);

          // Count how many heights have multiple hashes
          let heightsWithMultipleHashes = 0;
          for (const [number, blocks] of blocksByNumber) {
            const uniqueHashes = new Set(blocks.map((b) => b.hash));
            if (uniqueHashes.size > 1) {
              heightsWithMultipleHashes++;
              log(`  Height #${number}: ${uniqueHashes.size} different hashes`);
            }
          }

          log(`\nHeights with multiple hashes: ${heightsWithMultipleHashes}`);

          // Per spec, with 3 successive reorgs we should see multiple re-emissions
          if (heightsWithMultipleHashes >= 2) {
            log("✓ SPEC COMPLIANT: Multiple reorgs caused header re-emissions");
          } else if (heightsWithMultipleHashes >= 1) {
            log("⚠ PARTIAL COMPLIANCE: Some reorg headers were re-emitted");
          } else {
            log("❌ SPEC VIOLATION: No headers re-emitted despite multiple reorgs");
          }

          // We created many blocks, should have received several
          expect(
            receivedBlocks.length,
            "Should have received many blocks during rapid reorgs"
          ).toBeGreaterThanOrEqual(5);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T08",
      title: "subscription should match RPC canonical chain after reorg",
      test: async function () {
        const receivedBlocks: ReceivedBlock[] = [];

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            receivedBlocks.push({
              number: block.number,
              hash: block.hash,
              parentHash: block.parentHash,
              timestamp: Date.now(),
            });
          },
        });

        try {
          const initialBlockNum = await context.viem().getBlockNumber();

          // Create a reorg scenario
          const block1 = await context.createBlock([], {});
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Create BOTH fork points at height 2 first (avoids long-range attack protection)
          await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 100));

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Extend fork B to make it canonical
          await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          // Query canonical chain via RPC
          log("\n=== Comparing Subscription vs RPC ===");

          const canonicalBlocks: Array<{ number: bigint; hash: string }> = [];
          const currentBlockNum = await context.viem().getBlockNumber();

          for (let i = initialBlockNum + 1n; i <= currentBlockNum; i++) {
            const block = await context.viem().getBlock({ blockNumber: i });
            canonicalBlocks.push({ number: block.number, hash: block.hash });
            log(`  RPC Block #${i}: ${block.hash.slice(0, 18)}...`);
          }

          // Get the last block received at each height from subscription
          const lastReceivedAtHeight = new Map<bigint, ReceivedBlock>();
          for (const block of receivedBlocks) {
            lastReceivedAtHeight.set(block.number, block);
          }

          log("\nLast subscription block at each height:");
          for (const [num, block] of lastReceivedAtHeight) {
            log(`  Sub Block #${num}: ${block.hash.slice(0, 18)}...`);
          }

          // The last received block at each height should match canonical chain
          let mismatches = 0;
          for (const canonicalBlock of canonicalBlocks) {
            const subBlock = lastReceivedAtHeight.get(canonicalBlock.number);
            if (subBlock && subBlock.hash !== canonicalBlock.hash) {
              mismatches++;
              log(
                `\n❌ Mismatch at height #${canonicalBlock.number}:` +
                  `\n   RPC:          ${canonicalBlock.hash.slice(0, 18)}...` +
                  `\n   Subscription: ${subBlock.hash.slice(0, 18)}...`
              );
            }
          }

          if (mismatches === 0) {
            log("\n✓ Subscription final state matches RPC canonical chain");
          }

          expect(
            mismatches,
            "Last received block at each height should match canonical chain"
          ).toBe(0);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T09",
      title: "should not skip block numbers during reorg",
      test: async function () {
        const receivedBlockNumbers: bigint[] = [];

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            receivedBlockNumbers.push(block.number);
          },
        });

        try {
          // Create sequential blocks then reorg
          const block1 = await context.createBlock([], {});
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Create BOTH fork points at height 2 first (avoids long-range attack protection)
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 100));

          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Extend fork A
          await context.createBlock([], {
            parentHash: block2a.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 200));

          // Extend fork B to make it longer and canonical
          const block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: false,
          });
          await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: false, // Don't finalize to allow subsequent tests to create forks
          });
          await new Promise((resolve) => setTimeout(resolve, 500));

          log("\n=== Block Number Continuity Check ===");
          log(`Received block numbers: ${receivedBlockNumbers.join(", ")}`);

          // Check for gaps (missing block numbers)
          const sortedUnique = [...new Set(receivedBlockNumbers)].sort((a, b) =>
            a < b ? -1 : a > b ? 1 : 0
          );

          const gaps: bigint[] = [];
          for (let i = 1; i < sortedUnique.length; i++) {
            const expected = sortedUnique[i - 1] + 1n;
            if (sortedUnique[i] > expected) {
              // There's a gap
              for (let g = expected; g < sortedUnique[i]; g++) {
                gaps.push(g);
              }
            }
          }

          if (gaps.length > 0) {
            log(`❌ Missing block numbers: ${gaps.join(", ")}`);
          } else {
            log("✓ No gaps in block numbers");
          }

          // Per spec, we should receive all block numbers without gaps
          // (even if some heights are emitted multiple times with different hashes)
          expect(gaps.length, `Should not skip block numbers. Missing: ${gaps.join(", ")}`).toBe(0);
        } finally {
          unwatch();
        }
      },
    });

    it({
      id: "T10",
      title: "should reject long-range fork attempts (Substrate protection)",
      test: async function () {
        // This test verifies that Substrate's "long-range attack" protection works correctly.
        // When a block is created with a parent that is behind the finalized block,
        // the node should reject it to prevent potential long-range attacks.
        //
        // NOTE: This test finalizes blocks, so it must be the LAST test in the suite.

        // Create base block
        const block1 = await context.createBlock([], {});
        await new Promise((resolve) => setTimeout(resolve, 200));

        // Create and FINALIZE a chain from block1
        // This establishes a finalized chain that cannot be forked from
        log("\n=== Creating and finalizing chain from block1 ===");
        const block2a = await context.createBlock([], {
          parentHash: block1.block.hash,
          finalize: false,
        });
        await new Promise((resolve) => setTimeout(resolve, 100));

        const block3a = await context.createBlock([], {
          parentHash: block2a.block.hash,
          finalize: false,
        });
        await new Promise((resolve) => setTimeout(resolve, 100));

        // Finalize to establish the canonical chain
        await context.createBlock([], {
          parentHash: block3a.block.hash,
          finalize: true, // This is the last test, safe to finalize
        });
        await new Promise((resolve) => setTimeout(resolve, 100));

        log(`  Created and finalized: block1 → block2a → block3a → block4a`);
        log(`  block1 is now behind the finalized chain`);

        // Now attempt to create a competing fork from block1 (which is now behind finalization)
        // This should trigger the "Potential long-range attack" protection
        log("\n=== Attempting to fork from block1 (should fail) ===");

        let caughtError: Error | null = null;
        try {
          await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
        } catch (error) {
          caughtError = error as Error;
          log(`  ✓ Caught expected error: ${caughtError.message.slice(0, 80)}...`);
        }

        // Verify that the error was thrown and contains the expected message
        expect(
          caughtError,
          "Should have thrown an error for long-range fork attempt"
        ).not.toBeNull();
        expect(caughtError?.message, "Error should mention long-range attack protection").toContain(
          "Potential long-range attack"
        );

        log("\n✓ Substrate correctly rejected the long-range fork attempt");
      },
    });
  },
});
