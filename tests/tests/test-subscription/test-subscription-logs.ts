import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { Log } from "web3-core";

import { alith, ALITH_CONTRACT_ADDRESSES } from "../../util/accounts";
import { EnhancedWeb3, web3Subscribe } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Subscription - Logs", (context) => {
  let web3Ws: EnhancedWeb3;
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

    const { rawTx } = await createContract(context, "EventEmitter");
    await context.createBlock(rawTx);

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

  let subSingleAddPromise: Promise<Log>;
  let subMultiAddPromise: Promise<Log>;
  let subTopicPromise: Promise<Log>;
  let subTopicWildcardPromise: Promise<Log>;
  let subTopicListPromise: Promise<Log>;
  let subTopicCondPromise: Promise<Log>;
  let subTopicMultiCondPromise: Promise<Log>;
  let subTopicWildAndCondPromise: Promise<Log>;

  before("Setup: Create all subs and a block with transfer", async () => {
    web3Ws = await context.createWeb3("ws");

    const subSingleAdd = web3Subscribe(web3Ws, "logs", {
      address: ALITH_CONTRACT_ADDRESSES[0],
    });

    const subMultiAdd = web3Subscribe(web3Ws, "logs", {
      address: [
        ALITH_CONTRACT_ADDRESSES[3],
        ALITH_CONTRACT_ADDRESSES[2],
        ALITH_CONTRACT_ADDRESSES[1],
        ALITH_CONTRACT_ADDRESSES[0],
      ],
    });

    const subTopic = web3Subscribe(web3Ws, "logs", {
      topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
    });

    const subTopicWildcard = web3Subscribe(web3Ws, "logs", {
      topics: [null, "0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"],
    });

    const subTopicList = web3Subscribe(web3Ws, "logs", {
      topics: [
        ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
        ["0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"],
      ],
    });

    const subTopicCond = web3Subscribe(web3Ws, "logs", {
      topics: [
        "0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d",
        ["0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"],
      ],
    });

    const subTopicMultiCond = web3Subscribe(web3Ws, "logs", {
      topics: [
        "0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d",
        [
          "0x0000000000000000000000000000000000000000000000000000000000000000",
          "0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
        ],
      ],
    });

    const subTopicWildAndCond = web3Subscribe(web3Ws, "logs", {
      topics: [
        null,
        [
          "0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
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

    const subData = (sub: ReturnType<typeof web3Subscribe>) => {
      return new Promise<Log>((resolve) => {
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

    const { rawTx } = await createContract(context, "EventEmitter");
    await context.createBlock(rawTx);
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
    const { rawTx } = await createContract(context, "EventEmitter", {
      from: alith.address,
    });
    await context.createBlock(rawTx);

    const data = await new Promise((resolve) => {
      let result: Log = null;
      subscription.once("data", (d) => (result = d));
      setTimeout(() => resolve(result), 1000);
      // wait for 1 second to make sure a notification would have time to arrive.
      // (This one is not supposed to arrive because the transaction ran out of gas.)
    });

    subscription.unsubscribe();
    expect(data).to.be.null;
  });
});
