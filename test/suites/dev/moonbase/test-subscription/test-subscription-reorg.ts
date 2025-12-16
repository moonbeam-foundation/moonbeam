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

          // Finalize to lock the chain
          log("\n=== Finalizing Fork B (block 4b) ===");
          const block4b = await context.createBlock([], {
            parentHash: block3b.block.hash,
            finalize: true,
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

          // Create fork A
          const block2a = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          const forkBlockNum = block1Num + 1n; // Both 2a and 2b are at same height
          await new Promise((resolve) => setTimeout(resolve, 300));

          // Create fork B (same height as 2a)
          const block2b = await context.createBlock([], {
            parentHash: block1.block.hash,
            finalize: false,
          });
          await new Promise((resolve) => setTimeout(resolve, 300));

          // Extend fork B to trigger reorg
          const _block3b = await context.createBlock([], {
            parentHash: block2b.block.hash,
            finalize: true,
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

          // Per spec, we should receive BOTH blocks at height 2
          // This test documents the current (non-spec-compliant) behavior
          if (blocksAtHeight2.length < 2) {
            log("\n❌ SPEC VIOLATION: Did not receive both fork blocks at same height");
            log("   Expected: Both block2a and block2b should be delivered");
            log("   Per Ethereum spec, reorgs should emit all new headers for the new chain");
          } else {
            log("\n✓ SPEC COMPLIANT: Received multiple blocks at same height during reorg");
          }

          // This assertion documents expected spec behavior
          // It will fail on current implementation, proving the bug
          expect(
            blocksAtHeight2.length,
            `Expected to receive both fork blocks at height ${forkBlockNum} ` +
              `(per Ethereum spec), but only received ${blocksAtHeight2.length}. ` +
              `Block 2a received: ${receivedBlock2a}, Block 2b received: ${receivedBlock2b}`
          ).toBeGreaterThanOrEqual(2);
        } finally {
          unwatch();
        }
      },
    });
  },
});
