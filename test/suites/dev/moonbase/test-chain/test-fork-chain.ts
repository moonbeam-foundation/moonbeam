import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, TransactionTypes } from "@moonwall/cli";
import { createRawTransfer } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D020401",
  title: "Chain - Fork",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let randomAddress: `0x${string}`;

    beforeEach(async function () {
      const privateKey = generatePrivateKey();
      randomAddress = privateKeyToAccount(privateKey).address;
    });

    it({
      id: "T01",
      title: "should change best chain to the longest chain",
      test: async function () {
        // Creation of the best chain so far, with blocks 0-1-2
        await context.createBlock([], { finalize: false });
        await context.createBlock([], { finalize: false });

        // Lets grab the ethereum block hashes so far
        const ethHash1 = (await context.viem().getBlock({ blockNumber: 1n })).hash;
        const ethHash2 = (await context.viem().getBlock({ blockNumber: 2n })).hash;

        // Now lets fork the chain
        const currentHeight = await context.viem().getBlockNumber();

        // We start parenting to the genesis
        let parentHash = (await context.polkadotJs().rpc.chain.getBlockHash(0)).toString();
        for (let i = 0; i <= currentHeight; i++) {
          parentHash = (await context.createBlock([], { parentHash, finalize: false })).block.hash;
        }

        // We created at 1 block more than the previous best chain.
        // We should be in the best chain now
        expect(
          (await context.viem().getBlock({ blockNumber: 1n })).hash,
          "Ethereum blocks should have changed"
        ).to.not.equal(ethHash1);
        expect(
          (await context.viem().getBlock({ blockNumber: 2n })).hash,
          "Ethereum blocks should have changed"
        ).to.not.equal(ethHash2);
        expect((await context.viem().getBlock()).number).toBe(currentHeight + 1n);
      },
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 2}`,
        title: `should re-insert ${txnType} Tx from retracted fork on new canonical chain`,
        test: async function () {
          let parentHash = (await context.polkadotJs().rpc.chain.getBlockHash()).toString();

          //  Creation of the best chain so far, with blocks 0-1-2 and a transfer in block 2
          await context.createBlock([], { finalize: false });
          const { result } = await context.createBlock(
            createRawTransfer(context, randomAddress, 512),
            {
              finalize: false,
            }
          );
          const insertedTx = result!.hash as `0x${string}`;
          const retractedTx = await context.viem().getTransaction({ hash: insertedTx });
          expect(retractedTx).to.not.be.null;
          // Fork
          //   from: 0-1-2
          //   to  : 0-1b-2b-3b-4b-5b-6b-7b-8b-9b-10b
          // Create enough blocks to ensure the TX is re-scheduled and that chain is new best
          for (let i = 0; i < 10; i++) {
            parentHash = (await context.createBlock([], { parentHash, finalize: false })).block
              .hash;
            // TODO: investigate why ! Gives extra time (trouble with ci)
            await new Promise((resolve) => setTimeout(resolve, 100));
          }
          const finalTx = await context.viem().getTransaction({ hash: insertedTx });
          expect(
            finalTx.blockHash,
            "The Tx should have been inserted in the new best chain"
          ).to.not.be.eq(retractedTx.blockHash);
        },
      });
    }
  },
});
