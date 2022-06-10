import { expect } from "chai";
import { alith } from "../../util/accounts";

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
          (await context.polkadotApiParaone.query.system.account(alith.address.toString())) as any
        ).data.free.toBigInt() // TODO: fix type
      ).to.eq(1207825819614629174706176n);
    });
  }
);
