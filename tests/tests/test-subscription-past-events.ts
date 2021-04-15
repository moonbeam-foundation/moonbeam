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
  "Frontier RPC past events (Subscription)",
  `simple-specs.json`,
  (context) => {
    // Little helper to hack web3 that are not complete.
    function web3Subscribe(type: "pendingTransactions"): Subscription<string>;
    function web3Subscribe(type: "logs", params: {}): Subscription<Log>;
    function web3Subscribe(type: "newBlockHeaders" | "pendingTransactions" | "logs", params?: any) {
      return (context.web3.eth as any).subscribe(...arguments);
    }
    before(async function () {
      let first_subscription = web3Subscribe("pendingTransactions");
      await sendTransaction(context);
      await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        first_subscription.once("data", resolve);
      });
      first_subscription.unsubscribe();
      let second_subscription = web3Subscribe("logs", {});
      await sendTransaction(context);
      await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        second_subscription.once("data", resolve);
      });
      second_subscription.unsubscribe();
      let third_subscription = web3Subscribe("logs", {
        address: [
          "0xF8cef78E923919054037a1D03662bBD884fF4edf",
          "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
          "0x5c4242beB94dE30b922f57241f1D02f36e906915",
          "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
        ],
      });
      await sendTransaction(context);
      await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        third_subscription.once("data", resolve);
      });
      third_subscription.unsubscribe();
      let forth_subscription = web3Subscribe("logs", {
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      });
      await sendTransaction(context);
      await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        forth_subscription.once("data", resolve);
      });
      forth_subscription.unsubscribe();
    });
    it("should get past events #1: by topic", async function () {
      const subscription = web3Subscribe("logs", {
        fromBlock: "0x0",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      });

      const data = await new Promise((resolve) => {
        const data = [];
        subscription.on("data", function (d: any) {
          data.push(d);
          if (data.length == 4) resolve(data);
        });
      });
      subscription.unsubscribe();

      expect(data).to.not.be.empty;
    });

    it("should get past events #2: by address", async function () {
      const subscription = web3Subscribe("logs", {
        fromBlock: "0x0",
        address: "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
      });

      const data = await new Promise((resolve) => {
        const data = [];
        subscription.on("data", function (d: any) {
          data.push(d);
          if (data.length == 1) resolve(data);
        });
      });
      subscription.unsubscribe();

      expect(data).to.not.be.empty;
    });

    it("should get past events #3: by address + topic", async function () {
      const subscription = web3Subscribe("logs", {
        fromBlock: "0x0",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
        address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
      });

      const data = await new Promise((resolve) => {
        const data = [];
        subscription.on("data", function (d: any) {
          data.push(d);
          if (data.length == 1) resolve(data);
        });
      });
      subscription.unsubscribe();

      expect(data).to.not.be.empty;
    });

    it("should get past events #3: multiple addresses", async function () {
      const subscription = web3Subscribe("logs", {
        fromBlock: "0x0",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
        address: [
          "0xe573BCA813c741229ffB2488F7856C6cAa841041",
          "0xF8cef78E923919054037a1D03662bBD884fF4edf",
          "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
          "0x5c4242beB94dE30b922f57241f1D02f36e906915",
          "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
        ],
      });

      const data = await new Promise((resolve) => {
        const data = [];
        subscription.on("data", function (d: any) {
          data.push(d);
          if (data.length == 4) resolve(data);
        });
      });
      subscription.unsubscribe();

      expect(data).to.not.be.empty;
    });
  },
  "ws"
);
