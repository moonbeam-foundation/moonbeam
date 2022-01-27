import Keyring from "@polkadot/keyring";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";
import { BN, u8aToHex } from "@polkadot/util";

import { ALITH_PRIV_KEY, RANDOM_PRIV_KEY } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { customWeb3Request } from "../util/providers";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";

import { ParaId, XcmpMessageFormat } from "@polkadot/types/interfaces";

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

const sourceLocation = { XCM: { parents: 1, interior: { X1: { Parachain: foreign_para_id } } } };
const statemintLocation = {
  XCM: {
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

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerAsset(
          sourceLocation,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
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
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
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
      await context.polkadotApi.query.assets.account(assetId, alith.address)
    ).balance.toBigInt();

    expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerAsset
    // We register statemine with the new prefix
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerAsset(
          statemintLocation,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
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
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(statemintLocation, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should receive a 10 Statemine tokens to Alith with old prefix", async function () {
    // We are going to test that, using the prefix prior to
    // https://github.com/paritytech/cumulus/pull/831
    // we can receive the tokens on the assetId registed with the old prefix

    // Old prefix:
    // Parachain(Statemint parachain)
    // GeneralIndex(assetId being transferred)
    let xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: { X2: [{ Parachain: statemint_para_id }, { GeneralIndex: 0 }] },
                },
              },
              fun: { Fungible: new BN(10000000000000) },
            },
          ],
        },
        { ClearOrigin: null },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: new BN(1),
                  interior: { X2: [{ Parachain: statemint_para_id }, { GeneralIndex: 0 }] },
                },
              },
              fun: { Fungible: new BN(10000000000000) },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          DepositAsset: {
            assets: { Wild: "All" },
            maxAssets: new BN(1),
            beneficiary: {
              parents: 0,
              interior: { X1: { AccountKey20: { network: "Any", key: alith.address } } },
            },
          },
        },
      ],
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    );
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    );

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [
      statemint_para_id,
      totalMessage,
    ]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure the state has ALITH's foreign parachain tokens
    let alith_dot_balance = (
      await context.polkadotApi.query.assets.account(assetId, alith.address)
    ).balance.toBigInt();

    expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerAsset(
          statemintLocation,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
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
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(statemintLocation, 0)
      )
    );
    expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
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
    let xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    X3: [
                      { Parachain: statemint_para_id },
                      { PalletInstance: statemint_assets_pallet_instance },
                      { GeneralIndex: 0 },
                    ],
                  },
                },
              },
              fun: { Fungible: new BN(10000000000000) },
            },
          ],
        },
        { ClearOrigin: null },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: new BN(1),
                  interior: {
                    X3: [
                      { Parachain: statemint_para_id },
                      { PalletInstance: statemint_assets_pallet_instance },
                      { GeneralIndex: 0 },
                    ],
                  },
                },
              },
              fun: { Fungible: new BN(10000000000000) },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          DepositAsset: {
            assets: { Wild: "All" },
            maxAssets: new BN(1),
            beneficiary: {
              parents: 0,
              interior: { X1: { AccountKey20: { network: "Any", key: alith.address } } },
            },
          },
        },
      ],
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    );
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    );

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [
      statemint_para_id,
      totalMessage,
    ]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure the state has ALITH's foreign parachain tokens
    let alith_dot_balance = (
      await context.polkadotApi.query.assets.account(assetId, alith.address)
    ).balance.toBigInt();

    expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer of DEV", (context) => {
  let alith: KeyringPair;
  let random: KeyringPair;
  let paraId: ParaId;
  let transferredBalance;
  let sovereignAddress;

  before("Should send DEV to the parachain sovereign", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    random = keyringEth.addFromUri(RANDOM_PRIV_KEY, null, "ethereum");

    paraId = context.polkadotApi.createType("ParaId", 2000);
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    transferredBalance = new BN(100000000000000);

    // We first fund parachain 2000 sovreign account
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
    );
    let balance = (
      (await context.polkadotApi.query.system.account(sovereignAddress)) as any
    ).data.free.toBigInt();
    expect(balance.toString()).to.eq(transferredBalance.toString());
  });

  it("Should receive MOVR from para Id 2000", async function () {
    let ownParaId = (await context.polkadotApi.query.parachainInfo.parachainId()) as any;
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;
    // We are charging 100_000_000 weight for every XCM instruction
    // We are executing 4 instructions
    // 100_000_000 * 4 * 50000 = 20000000000000
    // We are charging 20 micro DEV for this operation
    // The rest should be going to the deposit account
    let xcmMessage = {
      V2: [
        {
          WithdrawAsset: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    X2: [{ Parachain: ownParaId }, { PalletInstance: balancesPalletIndex }],
                  },
                },
              },
              fun: { Fungible: transferredBalance },
            },
          ],
        },
        { ClearOrigin: null },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    X2: [{ Parachain: ownParaId }, { PalletInstance: balancesPalletIndex }],
                  },
                },
              },
              fun: { Fungible: transferredBalance },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          DepositAsset: {
            assets: { Wild: "All" },
            maxAssets: new BN(1),
            beneficiary: {
              parents: 0,
              interior: { X1: { AccountKey20: { network: "Any", key: random.address } } },
            },
          },
        },
      ],
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    );
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    );

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];

    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [2000, totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

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
    let expectedRandomBalance = 80000000000000n;
    expect(randomBalance.toString()).to.eq(expectedRandomBalance.toString());
  });
});
