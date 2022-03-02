import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN, bnToHex } from "@polkadot/util";
import { KeyringPair } from "@polkadot/keyring/types";

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
const sourceLocation = { XCM: { parents: 1, interior: "Here" } };
const newSourceLocation = { XCM: { parents: 1, interior: { X1: { Parachain: 1000 } } } };

describeDevMoonbeam("XCM - asset manager - register asset", (context) => {
  it("should be able to register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    const alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    const parachainOne = context.polkadotApi;
    // registerForeignAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.registerForeignAsset(
          sourceLocation,
          assetMetadata,
          new BN(1),
          true
        )
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
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = ((await parachainOne.query.assets.asset(assetId)) as any).unwrap();
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context, expect);
  });
});

describeDevMoonbeam("XCM - asset manager - register asset", (context) => {
  let assetId: string;
  let alith: KeyringPair;
  before("should be able to change existing asset type", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    const parachainOne = context.polkadotApi;
    // registerForeignAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.registerForeignAsset(
          sourceLocation,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );

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
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 1, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = ((await parachainOne.query.assets.asset(assetId)) as any).unwrap();
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context, expect);
  });

  it("should change the asset Id", async function () {
    // ChangeAssetType
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.changeExistingAssetType(assetId, newSourceLocation, 1)
      )
    );

    // asset_type
    let assetType = (await context.polkadotApi.query.assetManager.assetIdType(assetId)) as Object;

    // assetId
    let id = (
      (await context.polkadotApi.query.assetManager.assetTypeId(newSourceLocation)) as any
    ).unwrap();

    // asset units per second changed
    let assetUnitsPerSecond = (
      (await context.polkadotApi.query.assetManager.assetTypeUnitsPerSecond(
        newSourceLocation
      )) as any
    ).unwrap();

    // Supported assets
    let supportedAssets =
      (await context.polkadotApi.query.assetManager.supportedFeePaymentAssets()) as any;

    expect(assetUnitsPerSecond.toString()).to.eq(new BN(1).toString());
    expect(assetType.toString()).to.eq(JSON.stringify(newSourceLocation).toLowerCase());
    expect(bnToHex(id)).to.eq(assetId);
    expect(supportedAssets[0].toString()).to.eq(JSON.stringify(newSourceLocation).toLowerCase());
  });
});

describeDevMoonbeam("XCM - asset manager - register asset", (context) => {
  let assetId: string;
  let alith: KeyringPair;
  before("should be able to change existing asset type", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    const parachainOne = context.polkadotApi;
    // registerForeignAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.registerForeignAsset(
          sourceLocation,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );

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
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 1, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = ((await parachainOne.query.assets.asset(assetId)) as any).unwrap();
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context, expect);
  });

  it("should remove an asset from our supported fee payments", async function () {
    // ChangeAssetType
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.removeSupportedAsset(sourceLocation, 1)
      )
    );

    // assetId
    let id = (
      (await context.polkadotApi.query.assetManager.assetTypeId(sourceLocation)) as any
    ).unwrap();

    // asset units per second removed
    let assetUnitsPerSecond = (await context.polkadotApi.query.assetManager.assetTypeUnitsPerSecond(
      sourceLocation
    )) as any;

    // Supported assets should be 0
    let supportedAssets =
      (await context.polkadotApi.query.assetManager.supportedFeePaymentAssets()) as any;

    expect(assetUnitsPerSecond.isNone).to.eq(true);
    expect(bnToHex(id)).to.eq(assetId);
    // the asset should not be supported
    expect(supportedAssets.length).to.eq(0);
  });
});
