import { expect } from "chai";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_ACCOUNT } from "./constants";

const CONTRACT = require("./constants/TraceFilter.json");

describeWithMoonbeam("Moonbeam RPC (trace_filter)", `simple-specs.json`, (context) => {
  describe("Basic tracing tests", async () => {
    let new_hash;
    beforeEach(async function () {
      new_hash = await context.polkadotApi.rpc.chain.getBlockHash(0);

      this.timeout(15000);
      let current_height = await context.web3.eth.getBlockNumber();
      // We need to create as many blocks as the current longest chain plus 1 to allow for previously inserted tx to enter
      if (current_height != 0) {
        for (var i = 0; i < current_height; i++) {
          new_hash = (await createAndFinalizeBlock(context.polkadotApi, new_hash, false))[1];
        }
      }
    });
    it("Suceed transaction", async function () {
      this.timeout(150000);
      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          to: TEST_ACCOUNT,
          value: "0x200", // Must be higher than ExistentialDeposit (currently 0)
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      let current_height = await context.web3.eth.getBlockNumber();
      await createAndFinalizeBlock(context.polkadotApi, new_hash, false);

      // Perform RPC call.
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height + 1),
          toBlock: context.web3.utils.numberToHex(current_height + 1),
        },
      ]);

      expect(response.result.length).to.equal(1);
    });

    it("Replay reverting CREATE", async function () {
      this.timeout(150000);
      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          to: TEST_ACCOUNT,
          value: "0x200", // Must be higher than ExistentialDeposit (currently 0)
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

      let current_height = await context.web3.eth.getBlockNumber();

      console.log(current_height + 1);
      await createAndFinalizeBlock(context.polkadotApi, new_hash, false);

      // Perform RPC call.
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height + 1),
          toBlock: context.web3.utils.numberToHex(current_height + 1),
        },
      ]);

      expect(response.result.length).to.equal(1);
    });
  });
});
