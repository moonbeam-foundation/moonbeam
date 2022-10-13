import { expect } from "chai";
import { ALITH_ADDRESS, ALITH_GENESIS_TRANSFERABLE_BALANCE } from "../../util/accounts";

import { describeParachain } from "../../util/setup-para-tests";

describeParachain(
  "Ethereum RPC block tags",
  {
    parachain: {
      chain: "moonbase-local",
    },
  },
  (context) => {
    it("should support pending tag", async function () {
      this.timeout(150000);
      await context.waitBlocks(1);
      const expectedBalance = await context.web3.eth.getBalance(ALITH_ADDRESS, "pending");
      expect(BigInt(expectedBalance)).to.equal(ALITH_GENESIS_TRANSFERABLE_BALANCE);
    });
    it("should support merge tags in the parachain context", async function () {
      this.timeout(150000);
      await context.waitBlocks(3);
      // We waited for 3 more blocks, expect best block to be number 4.
      expect((await context.web3.eth.getBlock("latest")).number).to.equal(4);
      // `finalized` block to be 2.
      expect((await context.web3.eth.getBlock("finalized")).number).to.equal(2);
      // `safe` block to be an alias of `finalized` in the Polkadot context.
      expect((await context.web3.eth.getBlock("safe")).number).to.equal(2);
    });
  }
);
