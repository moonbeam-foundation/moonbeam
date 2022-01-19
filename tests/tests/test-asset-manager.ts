import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN } from "@polkadot/util";

import { ALITH_PRIV_KEY } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { verifyLatestBlockFees } from "../util/block";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const sourceLocation = { XCM: { X1: "Parent" } };

describeDevMoonbeam("XCM - asset manager - register asset", (context) => {
  it("should be able to register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    const alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    const parachainOne = context.polkadotApi;
    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1), true)
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
      parachainOne.tx.sudo.sudo(parachainOne.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0))
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = ((await parachainOne.query.assets.asset(assetId)) as any).unwrap();
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context, expect);
  });
});
