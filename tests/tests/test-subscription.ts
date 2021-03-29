import { expect } from "chai";
import { step } from "mocha-steps";
import { Subscription as Web3Subscription } from "web3-core-subscriptions";
import { BlockHeader } from "web3-eth";
import { Log } from "web3-core";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

// Extra type because web3 is not well typed
interface Subscription<T> extends Web3Subscription<T> {
  once: (type: "data" | "connected", handler: (data: T) => void) => Subscription<T>;
}

describeWithMoonbeam(
  "Frontier RPC (Subscription)",
  `simple-specs.json`,
  (context) => {
    let logs_generated = 0; // TODO: remove global variable used by tests

    const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
    const GENESIS_ACCOUNT_PRIVATE_KEY =
      "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

    // This reflects the measured gas cost of the transaction at this current point in time.
    // It has been known to fluctuate from release to release, so it may need adjustment.
    const EXPECTED_TRANSACTION_GAS_COST = 891328;

    const TEST_CONTRACT_BYTECODE =
      "0x608060405234801561001057600080fd5b50610041337fffffffffffffffffffffffffffffffffffffffffff" +
      "ffffffffffffffffffffff61004660201b60201c565b610291565b600073ffffffffffffffffffffffffffffff" +
      "ffffffffff168273ffffffffffffffffffffffffffffffffffffffff1614156100e9576040517f08c379a00000" +
      "0000000000000000000000000000000000000000000000000000815260040180806020018281038252601f8152" +
      "602001807f45524332303a206d696e7420746f20746865207a65726f2061646472657373008152506020019150" +
      "5060405180910390fd5b6101028160025461020960201b610c7c1790919060201c565b60028190555061015d81" +
      "6000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffff" +
      "ffff1681526020019081526020016000205461020960201b610c7c1790919060201c565b6000808473ffffffff" +
      "ffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190" +
      "8152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffff" +
      "ffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523" +
      "b3ef836040518082815260200191505060405180910390a35050565b6000808284019050838110156102875760" +
      "40517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001" +
      "8281038252601b8152602001807f536166654d6174683a206164646974696f6e206f766572666c6f7700000000" +
      "0081525060200191505060405180910390fd5b8091505092915050565b610e3a806102a06000396000f3fe6080" +
      "60405234801561001057600080fd5b50600436106100885760003560e01c806370a082311161005b57806370a0" +
      "8231146101fd578063a457c2d714610255578063a9059cbb146102bb578063dd62ed3e1461032157610088565b" +
      "8063095ea7b31461008d57806318160ddd146100f357806323b872dd146101115780633950935114610197575b" +
      "600080fd5b6100d9600480360360408110156100a357600080fd5b81019080803573ffffffffffffffffffffff" +
      "ffffffffffffffffff16906020019092919080359060200190929190505050610399565b604051808215151515" +
      "815260200191505060405180910390f35b6100fb6103b7565b6040518082815260200191505060405180910390" +
      "f35b61017d6004803603606081101561012757600080fd5b81019080803573ffffffffffffffffffffffffffff" +
      "ffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092" +
      "9190803590602001909291905050506103c1565b604051808215151515815260200191505060405180910390f3" +
      "5b6101e3600480360360408110156101ad57600080fd5b81019080803573ffffffffffffffffffffffffffffff" +
      "ffffffffff1690602001909291908035906020019092919050505061049a565b60405180821515151581526020" +
      "0191505060405180910390f35b61023f6004803603602081101561021357600080fd5b81019080803573ffffff" +
      "ffffffffffffffffffffffffffffffffff16906020019092919050505061054d565b6040518082815260200191" +
      "505060405180910390f35b6102a16004803603604081101561026b57600080fd5b81019080803573ffffffffff" +
      "ffffffffffffffffffffffffffffff16906020019092919080359060200190929190505050610595565b604051" +
      "808215151515815260200191505060405180910390f35b610307600480360360408110156102d157600080fd5b" +
      "81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803590602001909291" +
      "90505050610662565b604051808215151515815260200191505060405180910390f35b61038360048036036040" +
      "81101561033757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190" +
      "929190803573ffffffffffffffffffffffffffffffffffffffff169060200190929190505050610680565b6040" +
      "518082815260200191505060405180910390f35b60006103ad6103a6610707565b848461070f565b6001905092" +
      "915050565b6000600254905090565b60006103ce848484610906565b61048f846103da610707565b61048a8560" +
      "4051806060016040528060288152602001610d7060289139600160008b73ffffffffffffffffffffffffffffff" +
      "ffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020600061" +
      "0440610707565b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffff" +
      "ffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b61070f565b60019050" +
      "9392505050565b60006105436104a7610707565b8461053e85600160006104b8610707565b73ffffffffffffff" +
      "ffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260" +
      "200160002060008973ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffff" +
      "ffffffffffff16815260200190815260200160002054610c7c90919063ffffffff16565b61070f565b60019050" +
      "92915050565b60008060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffff" +
      "ffffffffffffffffffff168152602001908152602001600020549050919050565b60006106586105a261070756" +
      "5b8461065385604051806060016040528060258152602001610de160259139600160006105cc610707565b73ff" +
      "ffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260" +
      "200190815260200160002060008a73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffff" +
      "ffffffffffffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b6107" +
      "0f565b6001905092915050565b600061067661066f610707565b8484610906565b6001905092915050565b6000" +
      "600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffff" +
      "ffffff16815260200190815260200160002060008373ffffffffffffffffffffffffffffffffffffffff1673ff" +
      "ffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054905092915050565b6000" +
      "33905090565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffff" +
      "ffffffffffffff161415610795576040517f08c379a00000000000000000000000000000000000000000000000" +
      "00000000008152600401808060200182810382526024815260200180610dbd6024913960400191505060405180" +
      "910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffff" +
      "ffffffffffff16141561081b576040517f08c379a0000000000000000000000000000000000000000000000000" +
      "000000008152600401808060200182810382526022815260200180610d28602291396040019150506040518091" +
      "0390fd5b80600160008573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffff" +
      "ffffffffffffffff16815260200190815260200160002060008473ffffffffffffffffffffffffffffffffffff" +
      "ffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173" +
      "ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffffffff167f" +
      "8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92583604051808281526020019150" +
      "5060405180910390a3505050565b600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffff" +
      "ffffffffffffffffffffffffffffff16141561098c576040517f08c379a0000000000000000000000000000000" +
      "000000000000000000000000008152600401808060200182810382526025815260200180610d98602591396040" +
      "0191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273ffffffffffff" +
      "ffffffffffffffffffffffffffff161415610a12576040517f08c379a000000000000000000000000000000000" +
      "0000000000000000000000008152600401808060200182810382526023815260200180610d0560239139604001" +
      "91505060405180910390fd5b610a7d81604051806060016040528060268152602001610d4a6026913960008087" +
      "73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681" +
      "5260200190815260200160002054610bbc9092919063ffffffff16565b6000808573ffffffffffffffffffffff" +
      "ffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000" +
      "2081905550610b10816000808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffff" +
      "ffffffffffffffffffffff16815260200190815260200160002054610c7c90919063ffffffff16565b60008084" +
      "73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681" +
      "52602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffff" +
      "ffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f5" +
      "5a4df523b3ef836040518082815260200191505060405180910390a3505050565b6000838311158290610c6957" +
      "6040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020" +
      "01828103825283818151815260200191508051906020019080838360005b83811015610c2e5780820151818401" +
      "52602081019050610c13565b50505050905090810190601f168015610c5b578082038051600183602003610100" +
      "0a031916815260200191505b509250505060405180910390fd5b5060008385039050809150509392505050565b" +
      "600080828401905083811015610cfa576040517f08c379a0000000000000000000000000000000000000000000" +
      "00000000000000815260040180806020018281038252601b8152602001807f536166654d6174683a2061646469" +
      "74696f6e206f766572666c6f77000000000081525060200191505060405180910390fd5b809150509291505056" +
      "fe45524332303a207472616e7366657220746f20746865207a65726f206164647265737345524332303a206170" +
      "70726f766520746f20746865207a65726f206164647265737345524332303a207472616e7366657220616d6f75" +
      "6e7420657863656564732062616c616e636545524332303a207472616e7366657220616d6f756e742065786365" +
      "65647320616c6c6f77616e636545524332303a207472616e736665722066726f6d20746865207a65726f206164" +
      "647265737345524332303a20617070726f76652066726f6d20746865207a65726f206164647265737345524332" +
      "303a2064656372656173656420616c6c6f77616e63652062656c6f77207a65726fa265627a7a72315820c7a5ff" +
      "abf642bda14700b2de42f8c57b36621af020441df825de45fd2b3e1c5c64736f6c63430005100032";
    async function sendTransaction(context, extraData = {}) {
      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: TEST_CONTRACT_BYTECODE,
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

    // Little helper to hack web3 that are not complete.
    function web3Subscribe(type: "newBlockHeaders"): Subscription<BlockHeader>;
    function web3Subscribe(type: "pendingTransactions"): Subscription<string>;
    function web3Subscribe(type: "logs", params: {}): Subscription<Log>;
    function web3Subscribe(type: "newBlockHeaders" | "pendingTransactions" | "logs", params?: any) {
      return (context.web3.eth as any).subscribe(...arguments);
    }

    step("should connect", async function () {
      await createAndFinalizeBlock(context.polkadotApi);
      // @ts-ignore
      expect(context.web3.currentProvider.connected).to.equal(true);
    });

    step("should subscribe", async function () {
      const subscription = web3Subscribe("newBlockHeaders");
      const subscriptionId = await new Promise((resolve) =>
        subscription.once("connected", resolve)
      );

      subscription.unsubscribe();
      expect(subscriptionId).to.have.lengthOf(34);
    });

    step("should get newHeads stream", async function () {
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

    step("should get newPendingTransactions stream", async function () {
      const subscription = web3Subscribe("pendingTransactions");
      await new Promise((resolve) => subscription.once("connected", resolve));

      const tx = await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      logs_generated += 1; //TODO: this is wrong, test should not be dependant of other tests
      subscription.unsubscribe();

      expect(data).to.be.not.null;
      expect(tx["transactionHash"]).to.be.eq(data);
    });

    step("should subscribe to all logs", async function () {
      const subscription = web3Subscribe("logs", {});

      await new Promise((resolve) => {
        subscription.once("connected", resolve);
      });

      await sendTransaction(context);
      const data = await new Promise((resolve) => {
        createAndFinalizeBlock(context.polkadotApi);
        subscription.once("data", resolve);
      });
      logs_generated += 1;
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

    step("should subscribe to logs by address", async function () {
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

    step("should subscribe to logs by multiple addresses", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should subscribe to logs by topic", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should get past events #1: by topic", async function () {
      const subscription = web3Subscribe("logs", {
        fromBlock: "0x0",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      });

      const data = await new Promise((resolve) => {
        const data = [];
        subscription.on("data", function (d: any) {
          data.push(d);
          if (data.length == logs_generated) resolve(data);
        });
      });
      subscription.unsubscribe();

      expect(data).to.not.be.empty;
    });

    step("should get past events #2: by address", async function () {
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

    step("should get past events #3: by address + topic", async function () {
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

    step("should get past events #3: multiple addresses", async function () {
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
          if (data.length == logs_generated) resolve(data);
        });
      });
      subscription.unsubscribe();

      expect(data).to.not.be.empty;
    });

    step("should support topic wildcards", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should support single values wrapped around a sequence", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should support topic conditional parameters", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should support multiple topic conditional parameters", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should combine topic wildcards and conditional parameters", async function () {
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
      logs_generated += 1;
      subscription.unsubscribe();

      expect(data).to.not.be.null;
    });

    step("should not receive log when contract fails", async function () {
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
