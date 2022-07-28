import "@moonbeam-network/api-augment";

import { BN, bnToHex } from "@polkadot/util";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import {
  PARA_1000_SOURCE_LOCATION,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
} from "../../util/assets";
import { registerForeignAsset } from "../../util/xcm";
import { verifyLatestBlockFees } from "../../util/block";
import { GLMR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeDevMoonbeam("XCM - asset manager - foreign asset", (context) => {
  it("should be registerable and have unit per seconds set", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeam("XCM - asset manager - register local asset", (context) => {
  it("should be able to register a local asset", async function () {
    const parachainOne = context.polkadotApi;
    // registerForeignAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.registerLocalAsset(
          alith.address,
          alith.address,
          true,
          new BN(1)
        )
      )
    );
    // Look for assetId in events
    const assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // check asset in storage
    const registeredAsset = (await parachainOne.query.localAssets.asset(assetId)).unwrap();
    expect(registeredAsset.owner.toString()).to.eq(alith.address);

    // check deposit in storage
    const deposit = (await parachainOne.query.assetManager.localAssetDeposit(assetId)).unwrap();
    expect(deposit.creator.toString()).to.eq(alith.address);

    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeam("XCM - asset manager - Change existing asset", (context) => {
  let assetId: string;
  before("should be able to change existing asset type", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata,
      1
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context);
  });

  it("should change the asset Id", async function () {
    // ChangeAssetType
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.changeExistingAssetType(
          assetId,
          PARA_1000_SOURCE_LOCATION,
          1
        )
      )
    );

    // asset_type
    const assetType = (await context.polkadotApi.query.assetManager.assetIdType(assetId)) as Object;

    // assetId
    const id = (
      await context.polkadotApi.query.assetManager.assetTypeId(PARA_1000_SOURCE_LOCATION)
    ).unwrap();

    // asset units per second changed
    const assetUnitsPerSecond = (
      await context.polkadotApi.query.assetManager.assetTypeUnitsPerSecond(
        PARA_1000_SOURCE_LOCATION
      )
    ).unwrap();

    // Supported assets
    const supportedAssets =
      await context.polkadotApi.query.assetManager.supportedFeePaymentAssets();

    expect(assetUnitsPerSecond.toString()).to.eq(new BN(1).toString());
    expect(assetType.toString()).to.eq(JSON.stringify(PARA_1000_SOURCE_LOCATION).toLowerCase());
    expect(bnToHex(id)).to.eq(assetId);
    expect(supportedAssets[0].toString()).to.eq(
      JSON.stringify(PARA_1000_SOURCE_LOCATION).toLowerCase()
    );
  });
});

describeDevMoonbeam("XCM - asset manager - Remove asset from supported", (context) => {
  let assetId: string;
  before("should be able to change existing asset type", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata,
      1
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context);
  });

  it("should remove an asset from our supported fee payments", async function () {
    // ChangeAssetType
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.removeSupportedAsset(RELAY_SOURCE_LOCATION, 1)
      )
    );

    // assetId
    const id = (
      await context.polkadotApi.query.assetManager.assetTypeId(RELAY_SOURCE_LOCATION)
    ).unwrap();

    // asset units per second removed
    const assetUnitsPerSecond =
      await context.polkadotApi.query.assetManager.assetTypeUnitsPerSecond(RELAY_SOURCE_LOCATION);

    // Supported assets should be 0
    const supportedAssets =
      await context.polkadotApi.query.assetManager.supportedFeePaymentAssets();

    expect(assetUnitsPerSecond.isNone).to.eq(true);
    expect(bnToHex(id)).to.eq(assetId);
    // the asset should not be supported
    expect(supportedAssets.length).to.eq(0);
  });
});

describeDevMoonbeam("XCM - asset manager - destroy foreign asset", (context) => {
  let assetId: string;
  before("should be able to change existing asset type", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata,
      1
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toString()).to.eq(palletId);

    await verifyLatestBlockFees(context);
  });

  it("should be able to destroy a foreign asset through pallet-asset-manager", async function () {
    const assetDestroyWitness = context.polkadotApi.createType("PalletAssetsDestroyWitness", {
      accounts: 0,
      sufficients: 0,
      approvals: 0,
    });

    // Destroy foreign asset
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.destroyForeignAsset(assetId, assetDestroyWitness, 1)
      )
    );

    // assetId
    const id = await context.polkadotApi.query.assetManager.assetTypeId(RELAY_SOURCE_LOCATION);

    // asset units per second removed
    const assetUnitsPerSecond =
      await context.polkadotApi.query.assetManager.assetTypeUnitsPerSecond(RELAY_SOURCE_LOCATION);

    // Supported assets should be 0
    const supportedAssets =
      await context.polkadotApi.query.assetManager.supportedFeePaymentAssets();

    // assetDetails should have dissapeared
    const assetDetails = await context.polkadotApi.query.assets.asset(assetId);

    expect(assetUnitsPerSecond.isNone).to.eq(true);
    expect(id.isNone).to.eq(true);
    expect(assetDetails.isNone).to.eq(true);
    // the asset should not be supported
    expect(supportedAssets.length).to.eq(0);
  });
});

describeDevMoonbeam("XCM - asset manager - destroy local asset", (context) => {
  let assetId: string;
  before("should be able to change existing asset type", async function () {
    const parachainOne = context.polkadotApi;

    // Check ALITH has amount reserved
    const accountDetailsBefore = await parachainOne.query.system.account(alith.address);

    // registerAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      parachainOne.tx.sudo.sudo(
        parachainOne.tx.assetManager.registerLocalAsset(
          alith.address,
          alith.address,
          true,
          new BN(1)
        )
      )
    );

    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // check asset in storage
    const registeredAsset = (await parachainOne.query.localAssets.asset(assetId)).unwrap();
    expect(registeredAsset.owner.toString()).to.eq(alith.address);

    // Check ALITH has amount reserved
    const accountDetails = await parachainOne.query.system.account(alith.address);
    expect(accountDetails.data.reserved.toString()).to.eq(
      (accountDetailsBefore.data.reserved.toBigInt() + 100n * GLMR).toString()
    );
    await verifyLatestBlockFees(context);
  });

  it("should be able to destroy a local asset through pallet-asset-manager", async function () {
    const assetDestroyWitness = context.polkadotApi.createType("PalletAssetsDestroyWitness", {
      accounts: 0,
      sufficients: 0,
      approvals: 0,
    });

    // Reserved amount back to creator
    const accountDetailsBefore = await context.polkadotApi.query.system.account(alith.address);

    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.destroyLocalAsset(assetId, assetDestroyWitness)
      )
    );

    // assetDetails should have dissapeared
    let assetDetails = await context.polkadotApi.query.localAssets.asset(assetId);
    expect(assetDetails.isNone).to.eq(true);

    // Reserved amount back to creator
    const accountDetailsAfter = await context.polkadotApi.query.system.account(alith.address);

    // Amount should have decreased in one GLMR
    expect(accountDetailsAfter.data.reserved.toString()).to.eq(
      (accountDetailsBefore.data.reserved.toBigInt() - 100n * GLMR).toString()
    );

    // check deposit not in storage
    const deposit = await context.polkadotApi.query.assetManager.localAssetDeposit(assetId);
    expect(deposit.isNone).to.eq(true);
  });
});
