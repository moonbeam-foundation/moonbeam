import Keyring from "@polkadot/keyring";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";
import { BN, isUndefined, stringToU8a, u8aToHex } from "@polkadot/util";

import {
  ALITH,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TREASURY_ACCOUNT,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { describeParachain } from "../util/setup-para-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { numberToHex, stringToHex } from "web3-utils";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};
const sourceLocation = { XCM: { interior: { Here: null }, parents: new BN(1) } };

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
        parachainOne.tx.assetManager.registerAsset(sourceLocation, assetMetadata, new BN(1))
      )
    );
    // Look for assetId in events
    let assetId: string;
    eventsRegister.forEach((e) => {
      console.log(e.toHuman());
      let ev = e.toHuman();
      if (ev.section === "assetManager") {
        assetId = ev.data[0];
      }
    });
    assetId = assetId.replace(/,/g, "");

    // setAssetUnitsPerSecond
    const { events } = await createBlockWithExtrinsic(
      context,
      alith,
      parachainOne.tx.sudo.sudo(parachainOne.tx.assetManager.setAssetUnitsPerSecond(assetId, 0))
    );
    events.forEach((e) => console.log(e.toHuman()));
    expect(events[0].toHuman().method).to.eq("UnitsPerSecondChanged");
    expect(events[2].toHuman().method).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = await parachainOne.query.assets.asset(assetId);
    expect((registeredAsset.toHuman() as { owner: string }).owner).to.eq(palletId);
  });
});
