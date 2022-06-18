import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    // register index 0 for Alith
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.xcmTransactor.register(alith.address, 0)
      )
    );
  });

  it("allows to retrieve index through precompiles", async function () {
    const resp = await context.polkadotApi.query.xcmTransactor.indexToAccount(0);
    expect(resp.toString()).to.eq(alith.address);
  });
});
