import "@moonbeam-network/api-augment";

import { generateKeyringPair } from "../util/accounts";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam("Trace filter reorg", (context) => {
  it("succesfully reorg", async function () {
    const randomAccount = generateKeyringPair();
    this.timeout(150000000);

    // Create a first base block.
    const block1 = await context.createBlock([], {});

    // Create a first branch including a transaction.
    await context.createBlock(createTransfer(context, randomAccount.address, "0x200"), {
      parentHash: block1.block.hash,
      finalize: false,
    });
    // Contains nonce 0.

    // Create a branch. // nonce 1
    const block2a = await context.createBlock(
      createTransfer(context, randomAccount.address, "0x300", { nonce: 1 }),
      {
        parentHash: block1.block.hash,
        finalize: false,
      }
    );
    // Contains nonce 1.

    // Continue this new branch, it reorgs.
    //
    // This block doesn't contain the transaction with nonce 0. Reorg doesn't seems to add back
    // extrinsics into the pool.
    //
    // This block however will contain the transaction with nonce 1 but the
    // chain don't expect this nonce so the the Ethereum transaction in not executed.
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
    // With old tracer the nonce check was missing and thus the transaction was replayed, leading
    // to a mismatch and a crash when mapping the Frontier data.
    await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x07",
      },
    ]);
  });
});
