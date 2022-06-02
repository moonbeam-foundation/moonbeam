import Keyring from "@polkadot/keyring";
import { KeyringPair } from "@polkadot/keyring/types";
import { expect } from "chai";
import { BN, u8aToHex } from "@polkadot/util";

import {
  ALITH_PRIV_KEY,
  BALTATHAR_PRIVATE_KEY,
  GLMR,
  RANDOM_PRIV_KEY,
  TEST_ACCOUNT,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { customWeb3Request } from "../util/providers";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";

import { ParaId, XcmpMessageFormat } from "@polkadot/types/interfaces";
import { MultiLocation } from "@polkadot/types/interfaces";
import { deriveAddress } from "@substrate/txwrapper-core";

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

const statemintLocationAssetOne = {
  XCM: {
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
  let alith: KeyringPair;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerForeignAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
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
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0, 0)
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
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * FOREIGN_TOKEN);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerForeignAsset
    // We register statemine with the new prefix
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
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
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(statemintLocation, 0, 0)
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

  it("Should NOT receive a 10 Statemine tokens to Alith with old prefix", async function () {
    // We are going to test that, using the prefix prior to
    // https://github.com/paritytech/cumulus/pull/831
    // we cannot receive the tokens on the assetId registed with the old prefix

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
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

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
  let alith: KeyringPair;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerForeignAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
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
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(statemintLocation, 0, 0)
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
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

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
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

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

    paraId = context.polkadotApi.createType("ParaId", 2000) as any;
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    transferredBalance = 100000000000000n;

    // We first fund parachain 2000 sovreign account
    await createBlockWithExtrinsic(
      context,
      alith,
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
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];

    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [2000, totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

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
    let alith: KeyringPair;
    let random: KeyringPair;
    let paraId: ParaId;
    let transferredBalance;
    let sovereignAddress;

    before("Should send DEV to the parachain sovereign", async function () {
      const keyringEth = new Keyring({ type: "ethereum" });
      alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      random = keyringEth.addFromUri(RANDOM_PRIV_KEY, null, "ethereum");

      paraId = context.polkadotApi.createType("ParaId", 2000) as any;
      sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      transferredBalance = 100000000000000n;

      // We first fund parachain 2000 sovreign account
      await createBlockWithExtrinsic(
        context,
        alith,
        context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
      );
      let balance = (
        (await context.polkadotApi.query.system.account(sovereignAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it("Should receive MOVR from para Id 2000 with new reanchor logic", async function () {
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
                // This is the new reanchored logic
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
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
                  // This is the new reanchored logic
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
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
      ) as any;
      const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
        "XcmVersionedXcm",
        xcmMessage
      ) as any;

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
      expect(randomBalance).to.eq(expectedRandomBalance);
    });
  }
);

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;
  let alith: KeyringPair;
  let baltathar: KeyringPair;
  let paraId: ParaId;
  let transferredBalance;
  let sovereignAddress;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    baltathar = await keyringEth.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
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
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    transferredBalance = new BN(100000000000000);

    // mint asset
    await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.localAssets.mint(assetId, alith.address, transferredBalance)
    );

    paraId = context.polkadotApi.createType("ParaId", 2000) as any;
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund parachain 2000 sovreign account
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
    );

    // transfer to para Id sovereign to emulate having sent the tokens
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
    );
  });

  it("Should receive 10 Local Asset tokens and sufficent DEV to pay for fee", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
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
    let xcmMessage = {
      V2: [
        {
          WithdrawAsset: [
            {
              // This is the new reanchored logic
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
              },
              fun: { Fungible: transferredBalance },
            },
            {
              // This is the new reanchored logic
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X2: [{ PalletInstance: localAssetsPalletIndex }, { GeneralIndex: assetId }],
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
                // This is the new reanchored logic
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
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
            maxAssets: new BN(2),
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
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];

    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [foreign_para_id, totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

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
  let alith: KeyringPair;

  before(
    "Should Register two asset from same para but set unit per sec for one",
    async function () {
      const keyringEth = new Keyring({ type: "ethereum" });
      alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // registerAsset Asset 0
      // We register statemine with the new prefix
      const { events: eventsRegisterZero } = await createBlockWithExtrinsic(
        context,
        alith,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.registerForeignAsset(
            statemintLocation,
            assetMetadata,
            new BN(1),
            true
          )
        )
      );
      // Look for assetId in events
      eventsRegisterZero.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetIdZero = e.data[0].toHex();
        }
      });
      assetIdZero = assetIdZero.replace(/,/g, "");

      // registerAsset Asset 1
      // We register statemine with the new prefix
      const { events: eventsRegisterOne } = await createBlockWithExtrinsic(
        context,
        alith,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.registerForeignAsset(
            statemintLocationAssetOne,
            assetMetadata,
            new BN(1),
            true
          )
        )
      );
      // Look for assetId in events
      eventsRegisterOne.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetIdOne = e.data[0].toHex();
        }
      });
      assetIdOne = assetIdOne.replace(/,/g, "");

      // setAssetUnitsPerSecond.We only set it for statemintLocationAssetOne
      const { events } = await createBlockWithExtrinsic(
        context,
        alith,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(
            statemintLocationAssetOne,
            0,
            0
          )
        )
      );
      expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
      expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

      // check assets in storage
      const registeredAssetZero = (
        (await context.polkadotApi.query.assets.asset(assetIdZero)) as any
      ).unwrap();
      expect(registeredAssetZero.owner.toHex()).to.eq(palletId.toLowerCase());
      const registeredAssetOne = (
        (await context.polkadotApi.query.assets.asset(assetIdZero)) as any
      ).unwrap();
      expect(registeredAssetOne.owner.toHex()).to.eq(palletId.toLowerCase());
    }
  );

  it("Should receive 10 asset 0 tokens using statemint asset 1 as fee ", async function () {
    // We are going to test that, using one of them as fee payment (assetOne),
    // we can receive the other
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
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    X3: [
                      { Parachain: statemint_para_id },
                      { PalletInstance: statemint_assets_pallet_instance },
                      { GeneralIndex: 1 },
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
                      { GeneralIndex: 1 },
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
            maxAssets: new BN(2),
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
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

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
  let alith: KeyringPair;
  let baltathar: KeyringPair;
  let paraId: ParaId;
  let transferredBalance;
  let sovereignAddress;

  before("Should Register an asset and set unit per sec", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    baltathar = await keyringEth.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      alith,
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
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    transferredBalance = new BN(100000000000000);

    // mint asset
    await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.localAssets.mint(assetId, alith.address, transferredBalance)
    );

    paraId = context.polkadotApi.createType("ParaId", 2000) as any;
    sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund parachain 2000 sovreign account
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
    );

    // transfer to para Id sovereign to emulate having sent the tokens
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
    );
  });

  it("Should NOT receive 10 Local Assets and DEV for fee with old reanchor", async function () {
    let ownParaId = (await context.polkadotApi.query.parachainInfo.parachainId()) as any;
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
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
    let xcmMessage = {
      V2: [
        {
          WithdrawAsset: [
            {
              // This is the new reanchored logic
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
            {
              // This is the new reanchored logic
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    X3: [
                      { Parachain: ownParaId },
                      { PalletInstance: localAssetsPalletIndex },
                      { GeneralIndex: assetId },
                    ],
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
                // This is the new reanchored logic
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
            maxAssets: new BN(2),
            beneficiary: {
              parents: 0,
              interior: { X1: { AccountKey20: { network: "Any", key: baltathar.address } } },
            },
          },
        },
      ],
    };

    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [foreign_para_id, totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Old reanchor does not work anymore so no reception of tokens
    let baltatharLocalTokBalance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      baltathar.address
    )) as any;

    expect(baltatharLocalTokBalance.isNone).to.eq(true);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetIdZero: string;
  let alith: KeyringPair;

  before("Should register one asset without setting units per second", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // registerAsset Asset 0
    // We register statemine with the new prefix
    const { events: eventsRegisterZero } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          statemintLocation,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    eventsRegisterZero.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetIdZero = e.data[0].toHex();
      }
    });
    assetIdZero = assetIdZero.replace(/,/g, "");

    // check assets in storage
    const registeredAssetZero = (
      (await context.polkadotApi.query.assets.asset(assetIdZero)) as any
    ).unwrap();
    expect(registeredAssetZero.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should not receive 10 asset 0 tokens because fee not supported ", async function () {
    // We are going to test that, using one of them as fee payment (assetOne),
    // we can receive the other
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
            maxAssets: new BN(2),
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
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

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
    let alithAssetZeroBalance = (await context.polkadotApi.query.assets.account(
      assetIdZero,
      alith.address
    )) as any;

    expect(alithAssetZeroBalance.isNone).to.eq(true);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact", (context) => {
  let transferredBalance;
  let alith: KeyringPair;
  let DescendOriginAddress;
  let sendingAddress;

  before("Should receive transact action with DescendOrigin", async function () {
    const keyringEth = new Keyring({ type: "ethereum" });
    alith = keyringEth.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;

    const derivedMultiLocation: MultiLocation = context.polkadotApi.createType(
      "MultiLocation",
      JSON.parse(
        `{\
            "parents": 1,\
            "interior": {\
              "X2": [\
                { "Parachain": 1 },\
                { "AccountKey20": \
                  {\
                    "network": "Any",\
                    "key": "${allones}"\
                  } \
                }\
              ]\
            }\
          }`
      )
    );

    let toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 1n * GLMR;

    // We first fund parachain 2000 sovreign account
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
    );
    let balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it.only("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    let transferCall = context.polkadotApi.tx.balances.transfer(
      TEST_ACCOUNT,
      transferredBalance / 10n
    );
    let transferCallEncoded = transferCall?.method.toHex();
    // We are going to test that we can receive a transact operation from parachain 1
    // using descendOrigin first
    let xcmMessage = {
      V2: [
        {
          DescendOrigin: {
            X1: {
              AccountKey20: {
                network: "Any",
                key: sendingAddress,
              },
            },
          },
        },
        {
          WithdrawAsset: [
            {
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
              },
              fun: { Fungible: transferredBalance / 2n },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
              },
              fun: { Fungible: transferredBalance / 2n },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(1000000000),
            call: {
              encoded: transferCallEncoded,
            },
          },
        },
      ],
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure the state has ALITH's foreign parachain tokens
    let testAccountBalance = (
      (await context.polkadotApi.query.system.account(TEST_ACCOUNT)) as any
    ).data.free.toBigInt();

    expect(testAccountBalance).to.eq(transferredBalance / 10n);
  });
});
