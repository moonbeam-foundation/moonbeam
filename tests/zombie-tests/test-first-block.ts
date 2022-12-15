import { expect } from "chai";
import { alith, ALITH_GENESIS_TRANSFERABLE_BALANCE } from "../util/accounts";

import { describeZombienet } from "../util/setup-zombie-tests";

describeZombienet(
  `Check zombienet`,
  {
    parachain: {
      chain: "moonbase-local",
      binary: "local",
    },
  },
  (context) => {
    it("is running", async function () {
      await context.waitBlocks(1);
      expect(await context.web3.eth.getBalance(alith.address, 0)).to.equal(
        ALITH_GENESIS_TRANSFERABLE_BALANCE.toString()
      );
    });
  }
);
