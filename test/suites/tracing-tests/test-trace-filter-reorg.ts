import "@moonbeam-network/api-augment";
import { describeSuite, customDevRpcRequest } from "@moonwall/cli";
import { createEthersTransaction, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "T17",
  title: "Trace filter reorg",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "successfully reorg",
      timeout: 150000000,
      test: async function () {
        const randomAccount = generateKeyringPair();

        // Create a first base block.
        const block1 = await context.createBlock([], {});

        const rawSigned = await createEthersTransaction(context, {
          to: randomAccount.address,
          data: null,
          value: "0x200",
          gasLimit: 25000,
        });

        // Create a first branch including a transaction.
        await context.createBlock(rawSigned, {
          parentHash: block1.block.hash,
          finalize: false,
        });
        // Contains nonce 0.

        const rawSigned2 = await createEthersTransaction(context, {
          to: randomAccount.address,
          data: null,
          value: "0x300",
          gasLimit: 25000,
          nonce: 1,
        });

        // Create a branch. // nonce 1
        const block2a = await context.createBlock(rawSigned2, {
          parentHash: block1.block.hash,
          finalize: false,
        });
        // Contains nonce 1.

        // Continue this new branch, it reorgs.
        //
        // This block doesn't contain the transaction with nonce 0. Reorg doesn't seems to add back
        // extrinsics into the pool.
        //
        // This block however will contain the transaction with nonce 1 but the
        // chain don't expect this nonce so the Ethereum transaction in not executed.
        // However it is still in the list of extrinsics for this block.
        const block3a = await context.createBlock([], {
          parentHash: block2a.block.hash,
          finalize: false,
        });
        // Contains nonce 1 again !.

        // Additionnal blocks.
        const block4a = await context.createBlock([], {
          parentHash: block3a.block.hash,
          finalize: true,
        });
        // Contains nonce 0.

        const block5a = await context.createBlock([], {
          parentHash: block4a.block.hash,
          finalize: true,
        });
        // Contains nonce 1.

        const block6a = await context.createBlock([], {
          parentHash: block5a.block.hash,
          finalize: true,
        });

        await context.createBlock([], {
          parentHash: block6a.block.hash,
          finalize: true,
        });

        // Trace block 3a.
        // With old tracer the nonce check was missing and thus the transaction was replayed,
        // leading to a mismatch and a crash when mapping the Frontier data.
        await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x01",
            toBlock: "0x07",
          },
        ]);
      },
    });
  },
});
