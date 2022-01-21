import { expect } from "chai";
import { web3Subscribe } from "../../util/providers";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Subscription - Logs", (context) => {
  let web3Ws;
  before("Setup: Create empty block", async () => {
    web3Ws = await context.createWeb3("ws");
  });

  it("should send a notification on new transaction", async function () {
    const subscription = web3Subscribe(web3Ws, "logs", {});

    await new Promise((resolve) => {
      subscription.once("connected", resolve);
    });

    const dataPromise = new Promise((resolve) => {
      subscription.once("data", resolve);
    });

    const { rawTx } = await createContract(context, "SingleEventContract");
    await context.createBlock({
      transactions: [rawTx],
    });

    const data = await dataPromise;
    subscription.unsubscribe();

    const block = await context.web3.eth.getBlock("latest");
    expect(data).to.include({
      blockHash: block.hash,
      blockNumber: block.number,
      data: "0x",
      logIndex: 0,
      removed: false,
      transactionHash: block.transactions[0],
      transactionIndex: 0,
      transactionLogIndex: "0x0",
    });
  });
});

describeDevMoonbeam("Subscription - Logs", (context) => {
  let web3Ws;

  let subSingleAddPromise;
  let subMultiAddPromise;
  let subTopicPromise;
  let subTopicWildcardPromise;
  let subTopicListPromise;
  let subTopicCondPromise;
  let subTopicMultiCondPromise;
  let subTopicWildAndCondPromise;

  before("Setup: Create all subs and a block with transfer", async () => {
    web3Ws = await context.createWeb3("ws");

    const subSingleAdd = web3Subscribe(web3Ws, "logs", {
      address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
    });

    const subMultiAdd = web3Subscribe(web3Ws, "logs", {
      address: [
        "0xF8cef78E923919054037a1D03662bBD884fF4edf",
        "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
        "0x5c4242beB94dE30b922f57241f1D02f36e906915",
        "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
      ],
    });

    const subTopic = web3Subscribe(web3Ws, "logs", {
      topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
    });

    const subTopicWildcard = web3Subscribe(web3Ws, "logs", {
      topics: [null, "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"],
    });

    const subTopicList = web3Subscribe(web3Ws, "logs", {
      topics: [
        ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
        ["0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"],
      ],
    });

    const subTopicCond = web3Subscribe(web3Ws, "logs", {
      topics: [
        "0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d",
        ["0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"],
      ],
    });

    const subTopicMultiCond = web3Subscribe(web3Ws, "logs", {
      topics: [
        "0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d",
        [
          "0x0000000000000000000000000000000000000000000000000000000000000000",
          "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b",
        ],
      ],
    });

    const subTopicWildAndCond = web3Subscribe(web3Ws, "logs", {
      topics: [
        null,
        [
          "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b",
          "0x0000000000000000000000000000000000000000000000000000000000000000",
        ],
        null,
      ],
    });

    await Promise.all(
      [
        subSingleAdd,
        subMultiAdd,
        subTopic,
        subTopicWildcard,
        subTopicList,
        subTopicCond,
        subTopicMultiCond,
        subTopicWildAndCond,
      ].map((sub, index) => {
        new Promise((resolve) => {
          sub.once("connected", resolve);
        });
      })
    );

    const subData = (sub) => {
      return new Promise((resolve) => {
        sub.once("data", resolve);
      });
    };

    subSingleAddPromise = subData(subSingleAdd);
    subMultiAddPromise = subData(subMultiAdd);
    subTopicPromise = subData(subTopic);
    subTopicWildcardPromise = subData(subTopicWildcard);
    subTopicListPromise = subData(subTopicList);
    subTopicCondPromise = subData(subTopicCond);
    subTopicMultiCondPromise = subData(subTopicMultiCond);
    subTopicWildAndCondPromise = subData(subTopicWildAndCond);

    const { rawTx } = await createContract(context, "SingleEventContract");
    await context.createBlock({
      transactions: [rawTx],
    });
  });

  it("should be able to filter by address", async function () {
    const data = await subSingleAddPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should be able to filter by multiple addresses", async function () {
    const data = await subMultiAddPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should be able to filter by topic", async function () {
    const data = await subTopicPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should be able to filter by topic wildcards", async function () {
    const data = await subTopicWildcardPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should be able to filter by topic list", async function () {
    const data = await subTopicListPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should be able to filter by topic conditional parameters", async function () {
    const data = await subTopicCondPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should support multiple topic conditional parameters", async function () {
    const data = await subTopicMultiCondPromise;
    expect(data).to.include({ blockNumber: 1 });
  });

  it("should combine topic wildcards and conditional parameters", async function () {
    const data = await subTopicWildAndCondPromise;
    expect(data).to.include({ blockNumber: 1 });
  });
});

describeDevMoonbeam("Subscription - Reverted transaction", (context) => {
  // TODO: Telmo to verify if this statement is true
  it.skip("should not send logs", async function () {
    const web3Ws = await context.createWeb3("ws");
    const subscription = web3Subscribe(web3Ws, "logs", {});
    await new Promise((resolve) => {
      subscription.once("connected", resolve);
    });

    // Expected to fail because of not enough fund to pay the deployment
    const { rawTx } = await createContract(context, "SingleEventContract", {
      from: "0x1111111111111111111111111111111111111111",
    });
    await context.createBlock({
      transactions: [rawTx],
    });

    const data = await new Promise((resolve) => {
      let result = null;
      subscription.once("data", (d) => (result = d));
      setTimeout(() => resolve(result), 1000);
      // wait for 1 second to make sure a notification would have time to arrive.
      // (This one is not supposed to arrive because the transaction ran out of gas.)
    });

    subscription.unsubscribe();
    expect(data).to.be.null;
  });
});
