import "@moonbeam-network/api-augment";

import { BN } from "@polkadot/util";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { RELAY_SOURCE_LOCATION } from "../../util/assets";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};

describeDevMoonbeam("Mock XCM - downward transfer with non-triggered error handler", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // setAssetUnitsPerSecond
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
      )
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure that Alith does not receive 10 dot because there is no error", async function () {
    // BuyExecution does not charge for fees because we registered it for not doing so
    // But since there is no error, and the deposit is on the error handler, the assets will be trapped
    const xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          SetErrorHandler: [
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
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure ALITH did not reveive anything
    let alith_dot_balance = (await context.polkadotApi.query.localAssets.account(
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
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // setAssetUnitsPerSecond
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
      )
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure that Alith does receive 10 dot because there is error", async function () {
    // BuyExecution does not charge for fees because we registered it for not doing so
    // As a consequence the trapped assets will be entirely credited
    const xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          SetErrorHandler: [
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
        },
        // This forces an error
        {
          Trap: 0,
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];

    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    let alith_dot_balance = (
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
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // setAssetUnitsPerSecond
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
      )
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure Alith receives 10 dot with appendix and without error", async function () {
    // BuyExecution does not charge for fees because we registered it for not doing so
    // As a consequence the trapped assets will be entirely credited
    const xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          SetAppendix: [
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
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    let alith_dot_balance = (
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
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // setAssetUnitsPerSecond
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
      )
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
  });

  it("Should make sure Alith receives 10 dot with appendix and without error", async function () {
    // BuyExecution does not charge for fees because we registered it for not doing so
    // As a consequence the trapped assets will be entirely credited
    const xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          SetAppendix: [
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
        },
        {
          Trap: 0,
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    let alith_dot_balance = (
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
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetId = eventsRegister
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

    // setAssetUnitsPerSecond
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
      )
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

    // check asset in storage
    const registeredAsset = (
      (await context.polkadotApi.query.assets.asset(assetId)) as any
    ).unwrap();
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

    // BuyExecution does not charge for fees because we registered it for not doing so
    // But since there is no error, and the deposit is on the error handler, the assets will be trapped
    const xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          SetErrorHandler: [
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
        },
      ],
    };

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();

    // Make sure ALITH did not reveive anything
    let alith_dot_balance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      alith.address
    )) as any;

    expect(alith_dot_balance.isNone).to.be.true;
  });

  it("Should make sure that Alith receives claimed assets", async function () {
    // BuyExecution does not charge for fees because we registered it for not doing so
    // As a consequence the trapped assets will be entirely credited
    const xcmMessage = {
      V2: [
        {
          ClaimAsset: {
            assets: [
              {
                id: {
                  Concrete: {
                    parents: 1,
                    interior: {
                      Here: null,
                    },
                  },
                },
                fun: { Fungible: 10n * RELAY_TOKEN },
              },
            ],
            // Ticket seems to indicate the version of the assets
            ticket: {
              parents: 0,
              interior: { X1: { GeneralIndex: 2 } },
            },
          },
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 1,
                  interior: {
                    Here: null,
                  },
                },
              },
              fun: { Fungible: 10n * RELAY_TOKEN },
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

    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // You can provide a message, but if you don't a downward transfer is the default
    await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [totalMessage]);

    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure the state has ALITH's to DOT tokens
    let alith_dot_balance = (
      (await context.polkadotApi.query.assets.account(assetId, alith.address)) as any
    )
      .unwrap()
      ["balance"].toBigInt();

    expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
  });
});
