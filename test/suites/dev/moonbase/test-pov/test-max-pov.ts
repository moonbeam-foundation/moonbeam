import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { type Abi, encodeFunctionData } from "viem";

describeSuite({
  id: "D012703",
  title: "PoV size test - approaching maximum limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let storageFillerAddress: `0x${string}`;
    let storageFillerAbi: Abi;

    // Target a PoV size of approximately 7.5MB
    const TARGET_POV_MB = 7.5;
    const TARGET_POV_BYTES = Math.floor(TARGET_POV_MB * 1024 * 1024);

    // We'll create storage slots with large values
    const SLOT_SIZE = 24 * 1024; // 24KB per slot
    const NUM_SLOTS = 350; // Should give us ~8.4MB of raw storage data

    beforeAll(async () => {
      // Deploy a contract specifically designed to fill storage
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "StorageFiller");
      storageFillerAddress = contractAddress;
      storageFillerAbi = abi;

      // Fill storage with large values in separate transactions
      // This creates many storage slots with large values
      log(`Filling ${NUM_SLOTS} storage slots with ${SLOT_SIZE} bytes each...`);

      // Fill in batches to avoid transaction size limits
      const BATCH_SIZE = 10;
      for (let i = 0; i < NUM_SLOTS; i += BATCH_SIZE) {
        const batchSize = Math.min(BATCH_SIZE, NUM_SLOTS - i);
        const fillData = encodeFunctionData({
          abi: storageFillerAbi,
          functionName: "fillStorageBatch",
          args: [i, batchSize, SLOT_SIZE],
        });

        const tx = await createEthersTransaction(context, {
          to: storageFillerAddress,
          data: fillData,
          txnType: "eip1559",
          gasLimit: 15000000n,
        });

        await context.createBlock(tx);
        log(`Filled slots ${i} to ${i + batchSize - 1}`);
      }
    });

    it({
      id: "T01",
      title: "should generate a large PoV by accessing many storage slots",
      test: async function () {
        // Now create a transaction that modifies all these storage slots
        // This will force the inclusion of all storage proofs in the PoV
        const modifyData = encodeFunctionData({
          abi: storageFillerAbi,
          functionName: "modifyStorageBatch",
          args: [0, NUM_SLOTS],
        });

        const gasEstimate = await context.viem().estimateGas({
          account: ALITH_ADDRESS,
          to: storageFillerAddress,
          data: modifyData,
        });

        log(`Estimated gas for modifying all slots: ${gasEstimate}`);

        const rawSigned = await createEthersTransaction(context, {
          to: storageFillerAddress,
          data: modifyData,
          txnType: "eip1559",
          gasLimit: gasEstimate * 120n / 100n, // Add 20% buffer
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
            const modifyData = encodeFunctionData({
              abi: storageFillerAbi,
              functionName: "modifyStorageBatch",
              args: [0, count],
            });

            const gasEstimate = await context.viem().estimateGas({
              account: ALITH_ADDRESS,
              to: storageFillerAddress,
              data: modifyData,
            });

            const rawSigned = await createEthersTransaction(context, {
              to: storageFillerAddress,
              data: modifyData,
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
