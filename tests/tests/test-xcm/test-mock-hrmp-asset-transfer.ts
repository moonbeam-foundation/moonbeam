import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { ParaId } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";

import { alith, baltathar, generateKeyringPair } from "../../util/accounts";
import { PARA_2000_SOURCE_LOCATION } from "../../util/assets";
import {
  registerForeignAsset,
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  XcmFragment,
} from "../../util/xcm";
import { customWeb3Request } from "../../util/providers";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { expectOk } from "../../util/expect";

const FOREIGN_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const foreign_para_id = 2000;
const statemint_para_id = 1001;
const statemint_assets_pallet_instance = 50;

const assetMetadata = {
  name: "FOREIGN",
  symbol: "FOREIGN",
  decimals: new BN(12),
  isFrozen: false,
};
const STATEMINT_LOCATION = {
  Xcm: {
    parents: 1,
    interior: {
      X3: [
        { Parachain: statemint_para_id },
        { PalletInstance: statemint_assets_pallet_instance },
        { GeneralIndex: 0 },
      ],
    },
  },
};
const STATEMINT_ASSET_ONE_LOCATION = {
  Xcm: {
    parents: 1,
    interior: {
      X3: [
        { Parachain: statemint_para_id },
        { PalletInstance: statemint_assets_pallet_instance },
        { GeneralIndex: 1 },
      ],
    },
  },
};

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      PARA_2000_SOURCE_LOCATION,
      assetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should receive a horizontal transfer of 10 FOREIGNs to Alith", async function () {
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a horizontal transfer is the default
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [foreign_para_id, []]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure the state has ALITH's foreign parachain tokens
    let alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      .balance.toBigInt();

    expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      STATEMINT_LOCATION,
      assetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should NOT receive a 10 Statemine tokens to Alith with old prefix", async function () {
    // We are going to test that, using the prefix prior to
    // https://github.com/paritytech/cumulus/pull/831
    // we cannot receive the tokens on the assetId registed with the old prefix

    // Old prefix:
    // Parachain(Statemint parachain)
    // GeneralIndex(assetId being transferred)
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 1,
            interior: { X2: [{ Parachain: statemint_para_id }, { GeneralIndex: 0 }] },
          },
        ],
        fungible: 10000000000000n,
      },
      weight_limit: new BN(4000000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .clear_origin()
      .buy_execution()
      .deposit_asset()
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, statemint_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure the state has ALITH's foreign parachain tokens
    let alith_dot_balance = (await context.polkadotApi.query.assets.account(
      assetId,
      alith.address
    )) as any;

    // The message execution failed
    expect(alith_dot_balance.isNone).to.be.true;
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      STATEMINT_LOCATION,
      assetMetadata
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should receive a 10 Statemine tokens to Alith with new prefix", async function () {
    // We are going to test that, using the prefix after
    // https://github.com/paritytech/cumulus/pull/831
    // we can receive the tokens on the assetId registed with the old prefix

    // New prefix:
    // Parachain(Statemint parachain)
    // PalletInstance(Statemint assets pallet instance)
    // GeneralIndex(assetId being transferred)
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 1,
            interior: {
              X3: [
                { Parachain: statemint_para_id },
                { PalletInstance: statemint_assets_pallet_instance },
                { GeneralIndex: 0 },
              ],
            },
          },
        ],
        fungible: 10000000000000n,
      },
      weight_limit: new BN(4000000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .clear_origin()
      .buy_execution()
      .deposit_asset()
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, statemint_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure the state has ALITH's foreign parachain tokens
    expect(
      (await context.polkadotApi.query.assets.account(assetId, alith.address))
        .unwrap()
        .balance.toBigInt()
    ).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer of DEV", (context) => {
  let random: KeyringPair;
  let paraId: ParaId;
  let transferredBalance: bigint;
  let sovereignAddress: string;

  before("Should send DEV to the parachain sovereign", async function () {
    random = generateKeyringPair();
    paraId = context.polkadotApi.createType("ParaId", 2000) as any;
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    transferredBalance = 100000000000000n;

    // We first fund parachain 2000 sovreign account
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
    );
    let balance = (
      (await context.polkadotApi.query.system.account(sovereignAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should NOT receive MOVR from para Id 2000 with old reanchor", async function () {
    let ownParaId = (await context.polkadotApi.query.parachainInfo.parachainId()) as any;
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    // We are charging 100_000_000 weight for every XCM instruction
    // We are executing 4 instructions
    // 100_000_000 * 4 * 50000 = 20000000000000
    // We are charging 20 micro DEV for this operation
    // The rest should be going to the deposit account
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 1,
            interior: {
              X2: [{ Parachain: ownParaId }, { PalletInstance: balancesPalletIndex }],
            },
          },
        ],
        fungible: transferredBalance,
      },
      weight_limit: new BN(4000000000),
      beneficiary: random.address,
    })
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset()
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, foreign_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // The message should not have been succesfully executed, since old prefix is not supported
    // anymore
    let balance = (
      (await context.polkadotApi.query.system.account(sovereignAddress)) as any
    ).data.free.toBigInt();
    expect(balance.toString()).to.eq(transferredBalance.toString());

    // the random address does not receive anything
    let randomBalance = (
      (await context.polkadotApi.query.system.account(random.address)) as any
    ).data.free.toBigInt();
    expect(randomBalance).to.eq(0n);
  });
});

describeDevMoonbeam(
  "Mock XCM - receive horizontal transfer of DEV with new reanchor",
  (context) => {
    let random: KeyringPair;
    let paraId: ParaId;
    let transferredBalance: bigint;
    let sovereignAddress: string;

    before("Should send DEV to the parachain sovereign", async function () {
      random = generateKeyringPair();
      paraId = context.polkadotApi.createType("ParaId", 2000) as any;
      sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      transferredBalance = 100000000000000n;

      // We first fund parachain 2000 sovreign account
      await expectOk(
        context.createBlock(
          context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
        )
      );
      let balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it("Should receive MOVR from para Id 2000 with new reanchor logic", async function () {
      // Get Pallet balances index
      const metadata = await context.polkadotApi.rpc.state.getMetadata();
      const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
        (pallet) => pallet.name === "Balances"
      ).index;
      // We are charging 100_000_000 weight for every XCM instruction
      // We are executing 4 instructions
      // 200_000_000 * 4 * 50000 = 40000000000000
      // We are charging 40 micro DEV for this operation
      // The rest should be going to the deposit account
      const xcmMessage = new XcmFragment({
        fees: {
          multilocation: [
            {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
          ],
          fungible: transferredBalance,
        },
        weight_limit: new BN(8000000000),
        beneficiary: random.address,
      })
        .withdraw_asset()
        .clear_origin()
        .buy_execution()
        .deposit_asset()
        .as_v2();

      // Send an XCM and create block to execute it
      await injectHrmpMessageAndSeal(context, foreign_para_id, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      // We should expect sovereign balance to be 0, since we have transferred the full amount
      let balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance.toString()).to.eq(0n.toString());

      // In the case of the random address: we have transferred 100000000000000,
      // but 20000000000000 have been deducted
      // for weight payment
      let randomBalance = (
        (await context.polkadotApi.query.system.account(random.address)) as any
      ).data.free.toBigInt();
      let expectedRandomBalance = 60000000000000n;
      expect(randomBalance).to.eq(expectedRandomBalance);
    });
  }
);

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let paraId: ParaId;
  let transferredBalance: bigint;
  let sovereignAddress: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerLocalAsset(
          baltathar.address,
          baltathar.address,
          true,
          new BN(1)
        )
      )
    );

    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    transferredBalance = 100000000000000n;

    // mint asset
    await context.createBlock(
      context.polkadotApi.tx.localAssets
        .mint(assetId, alith.address, transferredBalance)
        .signAsync(baltathar)
    );

    paraId = context.polkadotApi.createType("ParaId", 2000) as any;
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
      )
    );

    // transfer to para Id sovereign to emulate having sent the tokens
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
      )
    );
  });

  it("Should receive 10 Local Asset tokens and sufficent DEV to pay for fee", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const localAssetsPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "LocalAssets";
      }
    ).index;

    // We are charging 100_000_000 weight for every XCM instruction
    // We are executing 4 instructions
    // 100_000_000 * 4 * 50000 = 20000000000000
    // We are charging 20 micro DEV for this operation
    // The rest should be going to the deposit account
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          {
            parents: 0,
            interior: {
              X2: [{ PalletInstance: localAssetsPalletIndex }, { GeneralIndex: assetId }],
            },
          },
        ],
        fungible: transferredBalance,
      },
      weight_limit: new BN(4000000000),
      beneficiary: alith.address,
    })
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset(2n)
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, foreign_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure the state has ALITH's LOCAL parachain tokens
    let alithLocalTokBalance = (
      (await context.polkadotApi.query.localAssets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alithLocalTokBalance.toString()).to.eq(transferredBalance.toString());
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetIdZero: string;
  let assetIdOne: string;

  before(
    "Should Register two asset from same para but set unit per sec for one",
    async function () {
      // registerForeignAsset 0
      const { registeredAssetId: registeredAssetIdZero, registeredAsset: registeredAssetZero } =
        await registerForeignAsset(context, STATEMINT_LOCATION, assetMetadata);
      assetIdZero = registeredAssetIdZero;
      // registerForeignAsset 1
      const {
        registeredAssetId: registeredAssetIdOne,
        events,
        registeredAsset: registeredAssetOne,
      } = await registerForeignAsset(context, STATEMINT_ASSET_ONE_LOCATION, assetMetadata, 0, 1);
      assetIdOne = registeredAssetIdOne;

      expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
      expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
      expect(registeredAssetZero.owner.toHex()).to.eq(palletId.toLowerCase());
      expect(registeredAssetOne.owner.toHex()).to.eq(palletId.toLowerCase());
    }
  );

  it("Should receive 10 asset 0 tokens using statemint asset 1 as fee ", async function () {
    // We are going to test that, using one of them as fee payment (assetOne),
    // we can receive the other
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 1,
            interior: {
              X3: [
                { Parachain: statemint_para_id },
                { PalletInstance: statemint_assets_pallet_instance },
                { GeneralIndex: 0 },
              ],
            },
          },
          {
            parents: 1,
            interior: {
              X3: [
                { Parachain: statemint_para_id },
                { PalletInstance: statemint_assets_pallet_instance },
                { GeneralIndex: 1 },
              ],
            },
          },
        ],
        fungible: 10000000000000n,
      },
      weight_limit: new BN(4000000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .clear_origin()
      .buy_execution(1) // buy execution with asset at index 1
      .deposit_asset(2n)
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, statemint_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure the state has ALITH's foreign parachain tokens
    let alithAssetZeroBalance = (
      (await context.polkadotApi.query.assets.account(assetIdZero, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alithAssetZeroBalance).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let paraId: ParaId;
  let transferredBalance: bigint;
  let sovereignAddress: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerLocalAsset(
          baltathar.address,
          baltathar.address,
          true,
          new BN(1)
        )
      )
    );

    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    transferredBalance = 100000000000000n;

    // mint asset
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.localAssets
          .mint(assetId, alith.address, transferredBalance)
          .signAsync(baltathar)
      )
    );

    paraId = context.polkadotApi.createType("ParaId", 2000) as any;
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
      )
    );

    // transfer to para Id sovereign to emulate having sent the tokens
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
      )
    );
  });

  it("Should NOT receive 10 Local Assets and DEV for fee with old reanchor", async function () {
    let ownParaId = (await context.polkadotApi.query.parachainInfo.parachainId()) as any;
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const localAssetsPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "LocalAssets";
      }
    ).index;

    // We are charging 100_000_000 weight for every XCM instruction
    // We are executing 4 instructions
    // 100_000_000 * 4 * 50000 = 20000000000000
    // We are charging 20 micro DEV for this operation
    // The rest should be going to the deposit account
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 1,
            interior: {
              X2: [{ Parachain: ownParaId }, { PalletInstance: balancesPalletIndex }],
            },
          },
          {
            parents: 1,
            interior: {
              X3: [
                { Parachain: ownParaId },
                { PalletInstance: localAssetsPalletIndex },
                { GeneralIndex: assetId },
              ],
            },
          },
        ],
        fungible: transferredBalance,
      },
      weight_limit: new BN(4000000000),
      beneficiary: baltathar.address,
    })
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset(2n)
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, foreign_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Old reanchor does not work anymore so no reception of tokens
    let baltatharLocalTokBalance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      baltathar.address
    )) as any;

    expect(baltatharLocalTokBalance.isNone).to.eq(true);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;

  before("Should register one asset without setting units per second", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      STATEMINT_LOCATION,
      assetMetadata
    );
    assetId = registeredAssetId;
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should not receive 10 asset 0 tokens because fee not supported ", async function () {
    // We are going to test that, using one of them as fee payment (assetOne),
    // we can receive the other
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [
          {
            parents: 1,
            interior: { X2: [{ Parachain: statemint_para_id }, { GeneralIndex: 0 }] },
          },
        ],
        fungible: 10000000000000n,
      },
      weight_limit: new BN(4000000000),
      beneficiary: alith.address,
    })
      .reserve_asset_deposited()
      .clear_origin()
      .buy_execution()
      .deposit_asset(2n)
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, statemint_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure the state has ALITH's foreign parachain tokens
    let alithAssetZeroBalance = (await context.polkadotApi.query.assets.account(
      assetId,
      alith.address
    )) as any;

    expect(alithAssetZeroBalance.isNone).to.eq(true);
  });
});
