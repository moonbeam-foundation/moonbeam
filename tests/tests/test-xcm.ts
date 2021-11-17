import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN } from "@polkadot/util";

import { ALITH_PRIV_KEY } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { createBlock } from "typescript";
import { customRequest } from "../tests/util";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};

//TODO Gorka showed me to think of this as { parents: 1, intherior: here }. Are these the same?
const sourceLocation = { XCM: { X1: "Parent" } };

describeDevMoonbeam("Mock XCM - receive downward transfer", (context) => {
  //TODO this entire block copied from another test, make sure it is the right params for DOTs
  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    const alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1))
      )
    );
    // Look for assetId in events
    let assetId: string;
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    // setAssetUnitsPerSecond
    const { events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = ((await context.polkadotApi.query.assets.asset(assetId)) as any).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should receive a downward transfer of 10 DOTs to Alith", async function () {
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customRequest(context.web3, "xcm_injectDownwardMessage", [[]]);

    // Create a block in which the XCM will be executed
    awat createBlock();

    // Make sure the state (and events?) has ALITH's to DOT tokens
    expect(context.polkadotApi.query.assets.accout(assetId, alith)).to.eq(10 * glmr);
  });
});
