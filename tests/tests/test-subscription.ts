import { expect } from "chai";
import { Subscription as Web3Subscription } from "web3-core-subscriptions";
import { BlockHeader } from "web3-eth";
import { Log } from "web3-core";

import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_SUBSCRIPTION_CONTRACT_BYTECODE,
} from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

// Extra type because web3 is not well typed
interface Subscription<T> extends Web3Subscription<T> {
  once: (type: "data" | "connected", handler: (data: T) => void) => Subscription<T>;
}

// This reflects the measured gas cost of the transaction at this current point in time.
// It has been known to fluctuate from release to release, so it may need adjustment.
const EXPECTED_TRANSACTION_GAS_COST = 891328;

async function sendTransaction(context, extraData = {}) {
  const tx = await context.web3.eth.accounts.signTransaction(
    {
      from: GENESIS_ACCOUNT,
      data: TEST_SUBSCRIPTION_CONTRACT_BYTECODE,
      value: "0x00",
      gasPrice: "0x01",
      gas: "0x" + EXPECTED_TRANSACTION_GAS_COST.toString(16),
      ...extraData,
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
  return tx;
}

describeWithMoonbeam(
  "Frontier RPC (Subscription)",
  `simple-specs.json`,
  (context) => {
    // Little helper to hack web3 that are not complete.
    function web3Subscribe(type: "newBlockHeaders"): Subscription<BlockHeader>;
    function web3Subscribe(type: "pendingTransactions"): Subscription<string>;
    function web3Subscribe(type: "logs", params: {}): Subscription<Log>;
    function web3Subscribe(type: "newBlockHeaders" | "pendingTransactions" | "logs", params?: any) {
      return (context.web3.eth as any).subscribe(...arguments);
    }

    before(async () => {
      await createAndFinalizeBlock(context.polkadotApi);
    });

    it("should connect", async function () {
      // @ts-ignore
      expect(context.web3.currentProvider.connected).to.equal(true);
    });

    it("should subscribe", async function () {
      const subscription = web3Subscribe("newBlockHeaders");
      const subscriptionId = await new Promise((resolve) =>
        subscription.once("connected", resolve)
      );

      subscription.unsubscribe();
      expect(subscriptionId).to.have.lengthOf(34);
    });

    it("should get newHeads stream", async function () {
      const subscription = web3Subscribe("newBlockHeaders");
      const data = await new Promise<BlockHeader>((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.include({
        author: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
        difficulty: "0",
        extraData: "0x",
        logsBloom: `0x${"0".repeat(512)}`,
        miner: "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b",
        receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      });
      expect((data as any).sealFields).to.eql([
        "0x0000000000000000000000000000000000000000000000000000000000000000",
        "0x0000000000000000",
      ]);
    });

    it("should get newPendingTransactions stream", async function () {
      const subscription = web3Subscribe("pendingTransactions");
      await new Promise((resolve) => subscription.once("connected", resolve));

      const tx = await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.be.not.null;
      expect(tx["transactionHash"]).to.be.eq(data);
    });

    it("should subscribe to all logs", async function () {
      const subscription = web3Subscribe("logs", {});

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      const block = await context.web3.eth.getBlock("latest");
      expect(data).to.include({
        blockHash: block.hash,
        blockNumber: block.number,
        data: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        logIndex: 0,
        removed: false,
        transactionHash: block.transactions[0],
        transactionIndex: 0,
        transactionLogIndex: "0x0",
      });
    });

    it("should subscribe to logs by address", async function () {
      const subscription = web3Subscribe("logs", {
        address: "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    it("should subscribe to logs by multiple addresses", async function () {
      const subscription = web3Subscribe("logs", {
        address: [
          "0xF8cef78E923919054037a1D03662bBD884fF4edf",
          "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
          "0x5c4242beB94dE30b922f57241f1D02f36e906915",
          "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
        ],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();
      expect(data).to.not.be.null;
    });

    it("should subscribe to logs by topic", async function () {
      const subscription = web3Subscribe("logs", {
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    it("should support topic wildcards", async function () {
      const subscription = web3Subscribe("logs", {
        topics: [null, "0x0000000000000000000000000000000000000000000000000000000000000000"],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });

      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });
    it("should support single values wrapped around a sequence", async function () {
      const subscription = web3Subscribe("logs", {
        topics: [
          ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
          ["0x0000000000000000000000000000000000000000000000000000000000000000"],
        ],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      const tx = await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });
    it("should support topic conditional parameters", async function () {
      const subscription = web3Subscribe("logs", {
        topics: [
          "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
          [
            "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
          ],
        ],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });
      const tx = await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    it("should support multiple topic conditional parameters", async function () {
      const subscription = web3Subscribe("logs", {
        topics: [
          "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
          [
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b",
          ],
          [
            "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
          ],
        ],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      const tx = await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    it("should combine topic wildcards and conditional parameters", async function () {
      const subscription = web3Subscribe("logs", {
        topics: [
          null,
          [
            "0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b",
            "0x0000000000000000000000000000000000000000000000000000000000000000",
          ],
          null,
        ],
      });

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    it("should not receive log when contract fails", async function () {
      const subscription = web3Subscribe("logs", {});

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context, {
        gas: "0x" + (EXPECTED_TRANSACTION_GAS_COST - 1).toString(16), // lower than expected by 1
      });
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        let result = null;
        subscription.once("data", (d) => (result = d));
        setTimeout(() => resolve(result), 1000);
        // wait for 1 second to make sure a notification would have time to arrive.
        // (This one is not supposed to arrive because the transaction ran out of gas.)
      });
      subscription.unsubscribe();
      expect(data).to.be.null;
    });
  },
  "ws"
);
