import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution, createTransfer } from "../util/transactions";
import { GENESIS_ACCOUNT, TEST_ACCOUNT } from "../util/constants";

describeDevMoonbeam("Trace filter reorg", (context) => {
  it("succesfully reorg", async () => {
    // Create a first base block.
    const { block: block1 } = await context.createBlock({});

    // Create a first branch including a transaction.
    const tx = await createTransfer(context.web3, TEST_ACCOUNT, "0x200");
    const { block: block2 } = await context.createBlock({
      parentHash: block1.hash,
      finalize: false,
      transactions: [tx],
    });

    // Create a new longer branch (reorg).
    const tx2 = await createTransfer(context.web3, TEST_ACCOUNT, "0x200");
    const { block: block2a } = await context.createBlock({
      parentHash: block1.hash,
      finalize: false,
      transactions: [tx2],
    });

    const { block: block3a } = await context.createBlock({
      parentHash: block2a.hash,
      finalize: false,
    });

    // Trace block 3a.
    // Currently this call crashes the node !!!
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x03",
      },
    ]);
    console.log(response);
  });
});
