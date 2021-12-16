import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import { ALITH, ALITH_PRIV_KEY } from "../util/constants";

describeDevMoonbeam("Precompiles - xcm transactor", (context) => {
  let sudoAccount;
  before("Setup genesis account and relay accounts", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    // register index 0 for Alith
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.xcmTransactor.register(ALITH, 0))
      .signAndSend(sudoAccount);
    await context.createBlock();
  });

  it("allows to retrieve index through precompiles", async function () {
    const resp = await context.polkadotApi.query.xcmTransactor.indexToAccount(0);
    expect(resp.toString()).to.eq(ALITH);
  });
});
