import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { type Abi, encodeFunctionData } from "viem";
import { getAllBlockEvents } from "../../../../helpers/expect";

describeSuite({
  id: "D012703",
  title: "PoV size test - approaching maximum limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageFillerAddress: `0x${string}`;
    let storageFillerAbi: Abi;

    // Target a PoV size of approximately 3.75MB
    const TARGET_POV_MB = 3.75;
    const TARGET_POV_BYTES = Math.floor(TARGET_POV_MB * 1024 * 1024);

    // We'll create storage slots with large values
    const SLOT_SIZE = 1 * 1024; // 1KB per slot
    const NUM_SLOTS = 3500; // Should give us ~3.5MB of raw storage data

    beforeAll(async () => {
      // Deploy a contract specifically designed to fill storage
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "StorageFiller");
      storageFillerAddress = contractAddress;
      storageFillerAbi = abi;

      // Fill storage with large values in separate transactions
      // This creates many storage slots with large values
      log(`Filling ${NUM_SLOTS} storage slots with ${SLOT_SIZE} bytes each...`);

      // Fill in batches to avoid transaction size limits
      const BATCH_SIZE = 100;
      for (let i = 0; i < NUM_SLOTS; i += BATCH_SIZE) {
        const batchSize = Math.min(BATCH_SIZE, NUM_SLOTS - i);
        const fillData = encodeFunctionData({
          abi: storageFillerAbi,
          functionName: "fillStorageBatch",
          args: [i, batchSize, SLOT_SIZE],
        });

        const gasEstimate = await context.viem().estimateGas({
          account: ALITH_ADDRESS,
          to: storageFillerAddress,
          data: fillData,
        });

        const tx = await createEthersTransaction(context, {
          to: storageFillerAddress,
          data: fillData,
          txnType: "eip1559",
          gasLimit: gasEstimate,
        });

        const { result, block } = await context.createBlock(tx);

        const events = await getAllBlockEvents(block.hash, context);
        events.forEach(({ event }) => expect(event.section.toString() !== "Error"));

        log(`Filled slots ${i} to ${i + batchSize - 1}`);
      }
    });

    it({
      id: "T01",
      title: "should generate a large PoV by accessing many storage slots",
      test: async function () {
        // Now create a transaction that reads all these storage slots
        // This will force the inclusion of all storage proofs in the PoV
        const readData = encodeFunctionData({
          abi: storageFillerAbi,
          functionName: "readStorageBatchBy384",
          args: [0, NUM_SLOTS],
        });

        const gasEstimate = await context.viem().estimateGas({
          account: ALITH_ADDRESS,
          to: storageFillerAddress,
          data: readData,
        });

        log(`Estimated gas for reading all slots: ${gasEstimate}`);

        const rawSigned = await createEthersTransaction(context, {
          to: storageFillerAddress,
          data: readData,
          txnType: "eip1559",
          // Add 20% buffer to estimate
          gasLimit: gasEstimate * (120n / 100n),
        });

        const { result, block } = await context.createBlock(rawSigned);
        const proofSize = block.proofSize ?? 0;

        log(`Block proof size: ${proofSize} bytes (${(proofSize / (1024 * 1024)).toFixed(2)} MB)`);
        log(`Transaction successful: ${result?.successful} `);

        // Check if PoV size is in the expected range
        expect(block.proofSize).toBeGreaterThanOrEqual(TARGET_POV_BYTES * 0.7);
        expect(result?.successful).to.equal(true);
      },
    });

    it({
      id: "T02",
      title: "should measure PoV size with incremental storage access",
      test: async function () {
        // Test with different numbers of slots to see how PoV size scales
        const slotCounts = [50, 100, 200, 300, NUM_SLOTS];

        for (const count of slotCounts) {
          try {
            const readData = encodeFunctionData({
              abi: storageFillerAbi,
              functionName: "readStorageBatch",
              args: [0, count],
            });

            const gasEstimate = await context.viem().estimateGas({
              account: ALITH_ADDRESS,
              to: storageFillerAddress,
              data: readData,
            });

            const rawSigned = await createEthersTransaction(context, {
              to: storageFillerAddress,
              data: readData,
              txnType: "eip1559",
              gasLimit: gasEstimate * 120n / 100n,
            });

            const { result, block } = await context.createBlock(rawSigned);
            const proofSize = block.proofSize ?? 0;

            log(`Slots: ${count}, PoV size: ${proofSize} bytes(${(proofSize / (1024 * 1024)).toFixed(2)} MB), Success: ${result?.successful} `);
          } catch (error) {
            log(`Slots: ${count}, Error: ${error.message} `);
          }
        }
      },
    });
  },
});
