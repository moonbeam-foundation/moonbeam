import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { type PublicClient, createPublicClient, webSocket } from "viem";

describeSuite({
  id: "D023506",
  title: "Subscription - Block headers during reorg",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let client: PublicClient;

    beforeAll(async () => {
      const wsUrl = context.viem().transport.url.replace("http", "ws");
      log(`Connecting to WebSocket: ${wsUrl}`);
      const transport = webSocket(wsUrl);
      client = createPublicClient({ transport });
    });

    it({
      id: "T01",
      title: "should deliver all canonical block headers during reorg",
      timeout: 60000,
      test: async function () {
        const receivedBlocks: Array<{
          number: bigint;
          hash: string;
          parentHash: string;
        }> = [];
        const receivedHashes = new Set<string>();

        // Start subscription
        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            log(`Received block #${block.number} (${block.hash?.slice(0, 18)}...)`);
            receivedBlocks.push({
              number: block.number!,
              hash: block.hash!,
              parentHash: block.parentHash,
            });
            receivedHashes.add(block.hash!);
          },
        });

        // Give subscription time to establish
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Create initial chain: genesis -> block1 -> block2
        log("Creating initial chain...");
        const block1 = await context.createBlock([], { finalize: false });
        log(`Created block1: ${block1.block.hash}`);

        const block2 = await context.createBlock([], { finalize: false });
        log(`Created block2: ${block2.block.hash}`);

        // Give time for notifications
        await new Promise((resolve) => setTimeout(resolve, 500));

        // Now create a fork from genesis that is longer
        // This will trigger a reorg
        log("Creating fork from genesis (will trigger reorg)...");
        let forkParent = (await context.polkadotJs().rpc.chain.getBlockHash(0)).toString();

        // Create 3 blocks on the fork (longer than the 2-block original chain)
        const forkBlocks: string[] = [];
        for (let i = 0; i < 3; i++) {
          const forkBlock = await context.createBlock([], {
            parentHash: forkParent,
            finalize: false,
          });
          forkParent = forkBlock.block.hash;
          forkBlocks.push(forkBlock.block.hash);
          log(`Created fork block ${i + 1}: ${forkBlock.block.hash}`);
          await new Promise((resolve) => setTimeout(resolve, 100));
        }

        // Give time for all notifications to arrive
        await new Promise((resolve) => setTimeout(resolve, 1000));

        // Stop subscription
        unwatch();

        // Log summary
        log("\n--- Summary ---");
        log(`Total blocks received: ${receivedBlocks.length}`);
        log(`Unique hashes: ${receivedHashes.size}`);

        // Analyze results
        const gaps: Array<{ expected: bigint; received: bigint }> = [];
        const missingParents: Array<{
          blockNumber: bigint;
          parentHash: string;
        }> = [];

        for (let i = 1; i < receivedBlocks.length; i++) {
          const prev = receivedBlocks[i - 1];
          const curr = receivedBlocks[i];

          // Check for gaps
          if (curr.number !== prev.number + 1n) {
            gaps.push({ expected: prev.number + 1n, received: curr.number });
            log(`GAP: Expected #${prev.number + 1n}, got #${curr.number}`);
          }

          // Check for missing parents (skip first few blocks as they reference genesis)
          if (i >= 2 && !receivedHashes.has(curr.parentHash)) {
            missingParents.push({
              blockNumber: curr.number,
              parentHash: curr.parentHash,
            });
            log(
              `MISSING PARENT: Block #${curr.number} references parent ` +
                `${curr.parentHash.slice(0, 18)}... which was never delivered`
            );
          }
        }

        // We expect to receive blocks during the reorg
        // The exact behavior depends on whether the fix is applied
        expect(
          receivedBlocks.length,
          "Should have received multiple blocks"
        ).toBeGreaterThanOrEqual(3);

        // Log the test expectation
        if (gaps.length > 0 || missingParents.length > 0) {
          log("\n--- ISSUE DETECTED ---");
          log(
            "The subscription skipped blocks or missed parent references during reorg."
          );
          log(
            "This is the bug described in: https://github.com/moonbeam-foundation/moonbeam/issues/3415"
          );
        } else {
          log("\n--- NO ISSUES ---");
          log("All blocks were delivered correctly during reorg.");
        }

        // These assertions will FAIL until the fix is applied
        // Comment them out to observe the bug, uncomment to verify the fix
        expect(
          gaps,
          `Subscription skipped blocks: ${gaps.map((g) => `expected #${g.expected}, got #${g.received}`).join("; ")}`
        ).toHaveLength(0);

        expect(
          missingParents,
          `Missing parent blocks: ${missingParents.length} blocks reference parents that were never delivered`
        ).toHaveLength(0);
      },
    });

    it({
      id: "T02",
      title: "should handle multiple consecutive reorgs",
      timeout: 60000,
      test: async function () {
        const receivedBlocks: Array<{
          number: bigint;
          hash: string;
          parentHash: string;
        }> = [];

        const unwatch = client.watchBlocks({
          onBlock: (block) => {
            log(`Received block #${block.number} (${block.hash?.slice(0, 18)}...)`);
            receivedBlocks.push({
              number: block.number!,
              hash: block.hash!,
              parentHash: block.parentHash,
            });
          },
        });

        await new Promise((resolve) => setTimeout(resolve, 500));

        // First chain: genesis -> A1 -> A2
        log("Creating chain A...");
        const genesisHash = (
          await context.polkadotJs().rpc.chain.getBlockHash(0)
        ).toString();

        let chainAParent = genesisHash;
        for (let i = 0; i < 2; i++) {
          const block = await context.createBlock([], {
            parentHash: chainAParent,
            finalize: false,
          });
          chainAParent = block.block.hash;
          log(`Chain A block ${i + 1}: ${block.block.hash}`);
        }
        await new Promise((resolve) => setTimeout(resolve, 300));

        // Second chain (fork): genesis -> B1 -> B2 -> B3
        log("Creating chain B (triggers reorg 1)...");
        let chainBParent = genesisHash;
        for (let i = 0; i < 3; i++) {
          const block = await context.createBlock([], {
            parentHash: chainBParent,
            finalize: false,
          });
          chainBParent = block.block.hash;
          log(`Chain B block ${i + 1}: ${block.block.hash}`);
        }
        await new Promise((resolve) => setTimeout(resolve, 300));

        // Third chain (another fork): genesis -> C1 -> C2 -> C3 -> C4
        log("Creating chain C (triggers reorg 2)...");
        let chainCParent = genesisHash;
        for (let i = 0; i < 4; i++) {
          const block = await context.createBlock([], {
            parentHash: chainCParent,
            finalize: false,
          });
          chainCParent = block.block.hash;
          log(`Chain C block ${i + 1}: ${block.block.hash}`);
        }
        await new Promise((resolve) => setTimeout(resolve, 500));

        unwatch();

        log(`\nTotal blocks received: ${receivedBlocks.length}`);
        log(`Block numbers: ${receivedBlocks.map((b) => b.number).join(", ")}`);

        // We should have received blocks from all three chains
        // as each became the canonical chain temporarily
        expect(
          receivedBlocks.length,
          "Should receive blocks from multiple chain switches"
        ).toBeGreaterThanOrEqual(4);
      },
    });
  },
});
