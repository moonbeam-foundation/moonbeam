import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution, createTransfer } from "../util/transactions";
import { GENESIS_ACCOUNT, TEST_ACCOUNT } from "../util/constants";

describeDevMoonbeam("Trace filter reorg", (context) => {
  it("succesfully reorg", async function () {
    this.timeout(150000000);

    // Create a first base block.
    console.log("Creating Block 1");
    const block1 = await context.createBlock({});

    // Create a first branch including a transaction.
    console.log("Creating Block 2");
    const tx = await createTransfer(context.web3, TEST_ACCOUNT, "0x200");
    const block2 = await context.createBlock({
      parentHash: block1.block.hash,
      finalize: false,
      transactions: [tx],
    });
    console.log(JSON.stringify(block2.txResults));

    // Create a new longer branch (reorg).
    console.log("Creating Block 2a");
    const tx2 = await createTransfer(context.web3, TEST_ACCOUNT, "0x200");
    const block2a = await context.createBlock({
      parentHash: block1.block.hash,
      finalize: false,
      transactions: [tx2],
    });
    console.log(JSON.stringify(block2a.txResults));

    console.log("Creating Block 3a");
    const block3a = await context.createBlock({
      parentHash: block2a.block.hash,
      finalize: false,
    });
    console.log(JSON.stringify(block3a.txResults));

    // Usefull values for debugging :
    console.log(`1: ${block1.block.hash}`); // 0xe8286c210e0b625ea17002701c52dd888af1bcc4fd6ec4c93f8d89a088efc48a
    console.log(`2: ${block2.block.hash}`); // 0x1072f26924ba47b406e468bd686d799fd7d614f5c0a2edc9b412797bba40373d
    console.log(`2a: ${block2a.block.hash}`); // 0x2837cd59136135c6f9b6bfbbae4e2833a415d84350eb44df06bb95cba630f174
    console.log(`3a: ${block3a.block.hash}`); // 0xc3ae5eff32b8d72bb2fd94f21f7a83246b1dfacad40533a459790783f97ba6b5

    // Trace block 3a.
    // Currently this call crashes the node !!!
    console.log("trace_filter !");
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x03",
      },
    ]);
    console.log(response);
  });
});
