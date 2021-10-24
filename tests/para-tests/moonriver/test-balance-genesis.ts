import { expect } from "chai";

import { ALITH, GENESIS_ACCOUNT, GENESIS_ACCOUNT_BALANCE } from "../../util/constants";
import { describeParachain } from "../../util/setup-para-tests";

const MOONRIVER_SUDO_ACCOUNT = "0xb728c13034c3b6c6447f399d25b097216a0081ea";

describeParachain("Balance genesis", { chain: "moonriver-local" }, (context) => {
  it("should be accessible through polkadotjs", async function () {
    expect(
      (await context.polkadotApiParaone.query.system.account(ALITH)).data.free.toBigInt()
    ).to.eq(1207825819614629174706176n);
  });
});
