import { expect } from "chai";
import { customWeb3Request, web3Subscribe } from "../util/providers";
import { BlockHeader } from "web3-eth";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";
import { COLLATOR_ACCOUNT } from "../util/constants";

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
    // TODO this should not be needed. test seems to fail when the block is created to quickly
    // after the subscription
    await new Promise((resolve) => setTimeout(resolve, 100));

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
      author: COLLATOR_ACCOUNT,
      difficulty: "0",
      extraData: "0x",
      logsBloom: `0x${"0".repeat(512)}`,
      miner: "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac",
      receiptsRoot: "0xb641ad10026d278e29908b40c13ab496f2d9b719fc9b788c0504a8e5a2347bf3",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      transactionsRoot: "0xf04aada99f08ad276fb593633bcb320bbbaf4499e9bfa8d59a7385e1772c6bcd",
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
