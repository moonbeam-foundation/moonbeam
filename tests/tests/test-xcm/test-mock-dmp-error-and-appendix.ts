import "@moonbeam-network/api-augment";

import { BN } from "@polkadot/util";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { RELAY_SOURCE_LOCATION, relayAssetMetadata } from "../../util/assets";
import { registerForeignAsset, XcmFragment } from "../../util/xcm";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeDevMoonbeam("Mock XCM - downward transfer with non-triggered error handler", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure that Alith does not receive 10 dot without error", async function () {
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
          fungible: 10n * RELAY_TOKEN,
        },
      ],
      weight_limit: new BN(500000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .buy_execution()
      // BuyExecution does not charge for fees because we registered it for not doing so
      // But since there is no error, and the deposit is on the error handler, the assets
      // will be trapped
      .with(function () {
        return this.set_error_handler_with([this.deposit_asset]);
      })
      .clear_origin()
      .as_v2();

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure ALITH did not reveive anything
    const alith_dot_balance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      alith.address
    )) as any;

    expect(alith_dot_balance.isNone).to.be.true;
  });
});

describeDevMoonbeam("Mock XCM - downward transfer with triggered error handler", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure that Alith does receive 10 dot because there is error", async function () {
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
          fungible: 10n * RELAY_TOKEN,
        },
      ],
      weight_limit: new BN(1000000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .buy_execution()
      // BuyExecution does not charge for fees because we registered it for not doing so
      // As a consequence the trapped assets will be entirely credited
      .with(function () {
        return this.set_error_handler_with([this.deposit_asset]);
      })
      .trap()
      .as_v2();

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];

    // Send RPC call to inject XCM message
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    const alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - downward transfer with always triggered appendix", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure Alith receives 10 dot with appendix and without error", async function () {
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
          fungible: 10n * RELAY_TOKEN,
        },
      ],
      weight_limit: new BN(800000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .buy_execution()
      // Set an appendix to be executed after the XCM message is executed. No matter if errors
      .with(function () {
        return this.set_appendix_with([this.deposit_asset]);
      })
      .as_v2();

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    const alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - downward transfer with always triggered appendix", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure Alith receives 10 dot with appendix and without error", async function () {
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
          fungible: 10n * RELAY_TOKEN,
        },
      ],
      weight_limit: new BN(1000000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .buy_execution()
      // BuyExecution does not charge for fees because we registered it for not doing so
      // As a consequence the trapped assets will be entirely credited
      // The goal is to show appendix runs even if there is an error
      .with(function () {
        return this.set_appendix_with([this.deposit_asset]);
      })
      .trap()
      .as_v2();

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    const alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - downward transfer claim trapped assets", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec and trap assets", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      relayAssetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

    // BuyExecution does not charge for fees because we registered it for not doing so
    // But since there is no error, and the deposit is on the error handler, the assets
    // will be trapped.
    // Goal is to trapp assets, so that later can be claimed
    // Since we only BuyExecution, but we do not do anything with the assets after that,
    // they are trapped
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
          fungible: 10n * RELAY_TOKEN,
        },
      ],
    })
      .reserve_asset_deposited()
      .buy_execution()

      .as_v2();

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure ALITH did not reveive anything
    const alith_dot_balance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      alith.address
    )) as any;

    expect(alith_dot_balance.isNone).to.be.true;
  });

  it("Should make sure that Alith receives claimed assets", async function () {
    const xcmMessage = new XcmFragment({
      assets: [
        {
          multilocation: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
          fungible: 10n * RELAY_TOKEN,
        },
      ],
      weight_limit: new BN(1000000000),
      beneficiary: alith.address,
    })
      // Claim assets that were previously trapped
      // assets: the assets that were trapped
      // ticket: the version of the assets (xcm version)
      .claim_asset()
      .buy_execution()
      // Deposit assets, this time correctly, on Alith
      .deposit_asset()
      .as_v2();

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];

    // Send RPC call to inject XCM message
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    const alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
  });
});
