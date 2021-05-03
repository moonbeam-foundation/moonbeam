import { expect } from "chai";
import { customWeb3Request, web3Subscribe } from "../util/providers";
import { BlockHeader } from "web3-eth";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam("Subscription", (context) => {
  let web3Ws;
  before("Setup: Create empty block", async () => {
    web3Ws = await context.createWeb3("ws");
  });

  it("should return a valid subscriptionId", async function () {
    const subscription = web3Subscribe(web3Ws, "newBlockHeaders");
    const subscriptionId = await new Promise((resolve) => subscription.once("connected", resolve));

    subscription.unsubscribe();
    expect(subscriptionId).to.have.lengthOf(34);
  });
});

describeDevMoonbeam("Subscription - Block headers", (context) => {
  let web3Ws;
  before("Setup: Create empty block", async () => {
    web3Ws = await context.createWeb3("ws");
  });

  it("should send notification on new block", async function () {
    const subscription = web3Subscribe(web3Ws, "newBlockHeaders");
    await new Promise((resolve) => subscription.once("connected", resolve));

    await context.createBlock({
      transactions: [
        await createTransfer(context.web3, "0x1111111111111111111111111111111111111111", 0),
      ],
    });

    const data = await new Promise<BlockHeader>((resolve) => {
      subscription.once("data", resolve);
    });
    subscription.unsubscribe();

    expect(data).to.include({
      author: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      difficulty: "0",
      extraData: "0x",
      logsBloom: `0x${"0".repeat(512)}`,
      miner: "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b",
      receiptsRoot: "0x3f9d4f18305cd0de20569ab8f7efb114f6374c65d0f02fbc80fd275317b1d375",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      transactionsRoot: "0xfe6c195567b1b64b0e1e48b79e75ee25fa56a23540b207f94d83c4dbb1835631",
    });
    expect((data as any).sealFields).to.eql([
      "0x0000000000000000000000000000000000000000000000000000000000000000",
      "0x0000000000000000",
    ]);
  });
});

describeDevMoonbeam("Subscription - Pending transactions", (context) => {
  let web3Ws;
  before("Setup: Create empty block", async () => {
    web3Ws = await context.createWeb3("ws");
  });

  // TODO: Inspect why it requires to produce a block to receive the notification
  it.skip("should send notification on new transaction", async function () {
    const subscription = web3Subscribe(web3Ws, "pendingTransactions");
    await new Promise((resolve) => subscription.once("connected", resolve));

    const dataP = new Promise((resolve) => {
      subscription.once("data", resolve);
    });

    const { result } = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      await createTransfer(context.web3, "0x1111111111111111111111111111111111111111", 0),
    ]);

    // This test passes if you produce the block
    // await context.createBlock();

    const data = await dataP;

    subscription.unsubscribe();

    expect(data).to.be.not.null;
    expect(result).to.be.eq(data);
  });
});
