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
    const block2 = await context.createBlock({
      parentHash: block1.block.hash,
      finalize: false,
      transactions: [tx],
    });

    // Create a branch.
    // TODO : The transaction seems to be inserted in this block but nonce is invalid on this
    // branch.
    const tx2 = await createTransfer(context.web3, TEST_ACCOUNT, "0x200"); // nonce 1
    const block2a = await context.createBlock({
      parentHash: block1.block.hash,
      finalize: false,
      transactions: [tx2],
    });

    // Continue this new branch, it reorgs.
    //
    // This block doesn't contain the transaction with nonce 0. Reorg doesn't seems to add back
    // extrinsics into the pool.
    //
    // This next block will contain the transaction with nonce 1 (because it's in the pool) but the
    // chain don't expect this nonce so the the Ethereum transaction in not executed. However it is
    // still in the list of extrinsics for this block.
    const block3a = await context.createBlock({
      parentHash: block2a.block.hash,
      finalize: false,
    });

    // Trace block 3a.
    // With old tracer the nonce check was missing and thus the transaction was replayed, leading
    // to a mismatch and a crash when mapping the Frontier data.
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x03",
      },
    ]);
  });
});
