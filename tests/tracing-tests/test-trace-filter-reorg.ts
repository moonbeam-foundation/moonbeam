import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution, createTransfer } from "../util/transactions";
import { GENESIS_ACCOUNT, TEST_ACCOUNT } from "../util/constants";

describeDevMoonbeam("Trace filter reorg", (context) => {
  it("succesfully reorg", async function () {
    this.timeout(150000000);

    // Create a first base block.
    const block1 = await context.createBlock({});

    // Create a first branch including a transaction.
    const tx = await createTransfer(context.web3, TEST_ACCOUNT, "0x200"); // nonce 0
    console.log("block 2");
    const block2 = await context.createBlock({
      parentHash: block1.block.hash,
      finalize: false,
      transactions: [tx],
    });
    // Contains nonce 0.

    // Create a branch.
    const tx2 = await createTransfer(context.web3, TEST_ACCOUNT, "0x300", { nonce: 1 }); // nonce 1
    console.log("block 2a");
    const block2a = await context.createBlock({
      parentHash: block1.block.hash,
      finalize: false,
      transactions: [tx2],
    });
    // Contains nonce 1.

    console.log("block 3a");
    // Continue this new branch, it reorgs.
    //
    // This block doesn't contain the transaction with nonce 0. Reorg doesn't seems to add back
    // extrinsics into the pool.
    //
    // This block however will contain the transaction with nonce 1 but the
    // chain don't expect this nonce so the the Ethereum transaction in not executed. However it is
    // still in the list of extrinsics for this block.
    const block3a = await context.createBlock({
      parentHash: block2a.block.hash,
      finalize: false,
    });
    // Contains nonce 1 again !.

    console.log("block 4a");
    // Additionnal blocks.
    const block4a = await context.createBlock({
      parentHash: block3a.block.hash,
      finalize: true,
    });
    // Contains nonce 0.

    console.log("block 5a");
    const block5a = await context.createBlock({
      parentHash: block4a.block.hash,
      finalize: true,
    });
    // Contains nonce 1.

    console.log("block 6a");
    const block6a = await context.createBlock({
      parentHash: block5a.block.hash,
      finalize: true,
    });

    console.log("block 7a");
    const block7a = await context.createBlock({
      parentHash: block6a.block.hash,
      finalize: true,
    });

    console.log("trace");
    // Trace block 3a.
    // With old tracer the nonce check was missing and thus the transaction was replayed, leading
    // to a mismatch and a crash when mapping the Frontier data.
    let response6a = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x07",
      },
    ]);

    console.log(JSON.stringify(response6a));
  });
});
