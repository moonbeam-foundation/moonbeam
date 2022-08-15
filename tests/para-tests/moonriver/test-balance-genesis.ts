import { expect } from "chai";
import { alith, ALITH_GENESIS_FREE_BALANCE } from "../../util/accounts";

import { describeParachain } from "../../util/setup-para-tests";

describeParachain(
  "Balance genesis",
  {
    parachain: {
      chain: "moonriver-local",
    },
  },
  (context) => {
    it("should be accessible through polkadotjs", async function () {
      expect(
        (
          await context.polkadotApiParaone.query.system.account(alith.address.toString())
        ).data.free.toBigInt()
      ).to.eq(ALITH_GENESIS_FREE_BALANCE);
    });
  }
);
