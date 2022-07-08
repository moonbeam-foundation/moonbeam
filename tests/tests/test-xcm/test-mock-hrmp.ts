import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { ParaId, XcmpMessageFormat } from "@polkadot/types/interfaces";
import { BN, u8aToHex, hexToU8a } from "@polkadot/util";
import { expect } from "chai";
import { ChaChaRng } from "randchacha";

import { alith, baltathar, generateKeyingPair } from "../../util/accounts";
import { PARA_2000_SOURCE_LOCATION } from "../../util/assets";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm } from "@polkadot/types/lookup";

import { createContract, createContractExecution } from "../../util/transactions";
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

// enum to mark how xcmp execution went
enum XcmpExecution {
  // it means the xcmp message was executed on_initialization
  InitializationExecutedPassingBarrier,
  // it means the xcmp message failed in the barrier check on_initialization
  InitializationExecutedNotPassingBarrier,
  // it means the xcmp was executed on_idle
  OnIdleExecutedPassingBarrier,
}

// Function to calculate how messages coming from different paras will be executed
export async function calculateShufflingAndExecution(
  context: DevTestContext,
  numParaMsgs: number,
  weightUsePerMessage: bigint,
  totalXcmpWeight: bigint
) {
  // the randomization is as follows
  // for every rand number, we do module number of paras
  // the given index is swaped with that obtained with the
  // random number

  let weightAvailable = 0n;
  let weightUsed = 0n;

  // we want to mimic the randomization process in the queue
  let indices = Array.from(Array(numParaMsgs).keys());
  let shouldItExecute = new Array(numParaMsgs).fill(false);

  let seed = await context.polkadotApi.query.system.parentHash();
  let rng = new ChaChaRng(seed);

  const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
  const apiAt = await context.polkadotApi.at(signedBlock.block.header.hash);
  const queueConfig = (await apiAt.query.xcmpQueue.queueConfig()) as any;
  let decay = queueConfig.weightRestrictDecay.toBigInt();
  let thresholdWeight = queueConfig.thresholdWeight.toBigInt();

  for (let i = 0; i < numParaMsgs; i++) {
    let rand = rng.nextU32();
    let j = rand % numParaMsgs;
    [indices[i], indices[j]] = [indices[j], indices[i]];

    // mimics the decay algorithm
    if (totalXcmpWeight - weightUsed > thresholdWeight) {
      if (weightAvailable != totalXcmpWeight) {
        weightAvailable += (totalXcmpWeight - weightAvailable) / (decay + 1n);
        if (weightAvailable + thresholdWeight > totalXcmpWeight) {
          weightAvailable = totalXcmpWeight;
        }
      }
      let weight_remaining = weightAvailable - weightUsed;

      if (weight_remaining < weightUsePerMessage) {
        shouldItExecute[i] = XcmpExecution.InitializationExecutedNotPassingBarrier;
      } else {
        shouldItExecute[i] = XcmpExecution.InitializationExecutedPassingBarrier;
        weightUsed += weightUsePerMessage;
      }
    } else {
      // we know this will execute on idle
      shouldItExecute[i] = XcmpExecution.OnIdleExecutedPassingBarrier;
    }
  }

  return [indices, shouldItExecute];
}

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  let assetId: string;

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          PARA_2000_SOURCE_LOCATION,
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
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(PARA_2000_SOURCE_LOCATION, 0, 0)
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
    // We register statemine with the new prefix
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          STATEMINT_LOCATION,
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
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(STATEMINT_LOCATION, 0, 0)
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
        { ClearOrigin: null as any },
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

  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const {
      result: { events: eventsRegister },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          STATEMINT_LOCATION,
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
        context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(STATEMINT_LOCATION, 0, 0)
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
        { ClearOrigin: null as any },
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
    const r = await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [
      statemint_para_id,
      totalMessage,
    ]);
    console.log(r);

    // Create a block in which the XCM will be executed
    await context.createBlock();

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
    random = generateKeyingPair();
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
        { ClearOrigin: null as any },
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
    let random: KeyringPair;
    let paraId: ParaId;
    let transferredBalance: bigint;
    let sovereignAddress: string;

    before("Should send DEV to the parachain sovereign", async function () {
      random = generateKeyingPair();
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

    it("Should receive MOVR from para Id 2000 with new reanchor logic", async function () {
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
          { ClearOrigin: null as any },
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
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
    );

    // transfer to para Id sovereign to emulate having sent the tokens
    await context.createBlock(
      context.polkadotApi.tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
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
        { ClearOrigin: null as any },
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

  before(
    "Should Register two asset from same para but set unit per sec for one",
    async function () {
      // registerAsset Asset 0
      // We register statemine with the new prefix
      const {
        result: { events: eventsRegisterZero },
      } = await context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.registerForeignAsset(
            STATEMINT_LOCATION,
            assetMetadata,
            new BN(1),
            true
          )
        )
      );
      // Look for assetId in events
      assetIdZero = eventsRegisterZero
        .find(({ event: { section } }) => section.toString() === "assetManager")
        .event.data[0].toHex()
        .replace(/,/g, "");

      // registerAsset Asset 1
      // We register statemine with the new prefix
      const {
        result: { events: eventsRegisterOne },
      } = await context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.registerForeignAsset(
            STATEMINT_ASSET_ONE_LOCATION,
            assetMetadata,
            new BN(1),
            true
          )
        )
      );
      // Look for assetId in events
      assetIdOne = eventsRegisterOne
        .find(({ event: { section } }) => section.toString() === "assetManager")
        .event.data[0].toHex()
        .replace(/,/g, "");

      // setAssetUnitsPerSecond.We only set it for statemintLocationAssetOne
      const {
        result: { events },
      } = await context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(
            STATEMINT_ASSET_ONE_LOCATION,
            0,
            0
          )
        )
      );
      expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
      expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");

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
        { ClearOrigin: null as any },
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
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(sovereignAddress, transferredBalance)
    );

    // transfer to para Id sovereign to emulate having sent the tokens
    await context.createBlock(
      context.polkadotApi.tx.localAssets.transfer(assetId, sovereignAddress, transferredBalance)
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
        { ClearOrigin: null as any },
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

  before("Should register one asset without setting units per second", async function () {
    // registerAsset Asset 0
    // We register statemine with the new prefix
    const {
      result: { events: eventsRegisterZero },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.registerForeignAsset(
          STATEMINT_LOCATION,
          assetMetadata,
          new BN(1),
          true
        )
      )
    );
    // Look for assetId in events
    assetIdZero = eventsRegisterZero
      .find(({ event: { section } }) => section.toString() === "assetManager")
      .event.data[0].toHex()
      .replace(/,/g, "");

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
        { ClearOrigin: null as any },
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
  let DescendOriginAddress;
  let sendingAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;
    random = generateKeyingPair();
    const derivedMultiLocation = context.polkadotApi.createType(
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

    const toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 1000000000000000000n;

    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
    );
    const balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const transferCall = context.polkadotApi.tx.balances.transfer(
      random.address,
      transferredBalance / 10n
    );
    const transferCallEncoded = transferCall?.method.toHex();
    // We are going to test that we can receive a transact operation from parachain 1
    // using descendOrigin first
    const xcmMessage = {
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
    const testAccountBalance = (
      await context.polkadotApi.query.system.account(random.address)
    ).data.free.toBigInt();

    expect(testAccountBalance).to.eq(transferredBalance / 10n);
  });
});
describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test that XCMP is executed randomized and until exhausted", async function () {
    this.timeout(120000);
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    // lets work with restrict decay 0 for now
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.xcmpQueue.updateWeightRestrictDecay(0)
      )
    );

    const numParaMsgs = 50;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalXcmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalXcmpWeight * BigInt(2)) / BigInt(numParaMsgs);

    const weightPerXcmInst = 100_000_000n;
    // Now we need to construct the message. This needs to:
    // - pass barrier (withdraw + clearOrigin*n buyExecution)
    // - fail in buyExecution, so that the previous instruction weights are counted
    // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
    // how many clearOrigins do we need to append?

    // In this case we want to never reach the thresholdLimit, to make sure we execute every
    // single messages.
    const clearOriginsPerMessage = (weightPerMessage - weightPerXcmInst) / weightPerXcmInst;

    const instructions = [
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
            fun: { Fungible: 1 },
          },
        ],
      },
      ...Array(Number(clearOriginsPerMessage)).fill({
        ClearOrigin: null,
      }),
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
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const xcmMessage = {
      V2: instructions,
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

    // We want these isntructions to fail in BuyExecution. That means
    // WithdrawAsset needs to work. The only way for this to work
    // is to fund each sovereign account
    for (let i = 0; i < numParaMsgs; i++) {
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      // We first fund each sovereign account with 100
      // we will only withdraw 1, so no problem on this
      await context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n));
    }

    // now we start injecting messages
    // one per para
    for (let i = 0; i < numParaMsgs; i++) {
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [i + 1, totalMessage]);
    }

    await context.createBlock();

    // all the withdraws + clear origins `buyExecution
    const weightUsePerMessage = (clearOriginsPerMessage + 2n) * 100_000_000n;

    const result = await calculateShufflingAndExecution(
      context,
      numParaMsgs,
      weightUsePerMessage,
      totalXcmpWeight
    );

    const indices = result[0];
    const shouldItExecute = result[1];

    // assert we dont have on_idle execution
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedPassingBarrier) > -1).to.be
      .true;
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedNotPassingBarrier) > -1).to
      .be.true;
    expect(shouldItExecute.indexOf(XcmpExecution.OnIdleExecutedPassingBarrier) > -1).to.be.false;

    // check balances
    for (let i = 0; i < numParaMsgs; i++) {
      // we need to check the randomization works. We have the shuffleing
      // and the amount executed, we need to make sure the balances
      // corresponding to the first executedCount shuffled indices
      // has one less unit of token
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      let balance = await context.polkadotApi.query.system.account(sovereignAddress);

      if (
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.InitializationExecutedPassingBarrier ||
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.OnIdleExecutedPassingBarrier
      ) {
        expect(balance.data.free.toBigInt()).to.eq(99n);
      } else {
        expect(balance.data.free.toBigInt()).to.eq(100n);
      }
    }
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test XCMP with decay randomized until exhausted, then Onidle", async function () {
    this.timeout(120000);
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const numParaMsgs = 50;
    // let's target half of then being executed

    // xcmp reserved is BLOCK/4
    const totalXcmpWeight =
      context.polkadotApi.consts.system.blockWeights.maxBlock.toBigInt() / BigInt(4);

    // we want half of numParaMsgs to be executed. That give us how much each message weights
    const weightPerMessage = (totalXcmpWeight * BigInt(2)) / BigInt(numParaMsgs);

    const weightPerXcmInst = 100_000_000n;
    // Now we need to construct the message. This needs to:
    // - pass barrier (withdraw + clearOrigin*n buyExecution)
    // - fail in buyExecution, so that the previous instruction weights are counted
    // we know at least 2 instructions are needed per message (withdrawAsset + buyExecution)
    // how many clearOrigins do we need to append?

    // we will bias this number. The reason is we want to test the decay, and therefore we need
    // an unbalanced number of messages executed. We specifically need that at some point
    // we get out of the loop of the execution (we reach the threshold limit), to then
    // go on idle

    // for that reason, we multiply times 2
    const clearOriginsPerMessage = (weightPerMessage - weightPerXcmInst * 2n) / weightPerXcmInst;

    const instructions = [
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
            fun: { Fungible: 1 },
          },
        ],
      },
      ...Array(Number(clearOriginsPerMessage)).fill({
        ClearOrigin: null,
      }),
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
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    let xcmMessage = {
      V2: instructions,
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

    // We want these isntructions to fail in BuyExecution. That means
    // WithdrawAsset needs to work. The only way for this to work
    // is to fund each sovereign account
    for (let i = 0; i < numParaMsgs; i++) {
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      // We first fund each sovereign account with 100
      // we will only withdraw 1, so no problem on this
      await context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n));
    }

    // now we start injecting messages
    // one per para
    for (let i = 0; i < numParaMsgs; i++) {
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [i + 1, totalMessage]);
    }

    await context.createBlock();

    // all the withdraws + clear origins `buyExecution
    const weightUsePerMessage = (clearOriginsPerMessage + 2n) * 100_000_000n;

    // in this case, we have some that will execute on_initialize
    // some that will fail the execution
    // and some that will execute on_idle
    const result = await calculateShufflingAndExecution(
      context,
      numParaMsgs,
      weightUsePerMessage,
      totalXcmpWeight
    );

    const indices = result[0];
    const shouldItExecute = result[1];

    // assert we have all kinds of execution
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedPassingBarrier) > -1).to.be
      .true;
    expect(shouldItExecute.indexOf(XcmpExecution.InitializationExecutedNotPassingBarrier) > -1).to
      .be.true;
    expect(shouldItExecute.indexOf(XcmpExecution.OnIdleExecutedPassingBarrier) > -1).to.be.true;

    // check balances
    for (let i = 0; i < numParaMsgs; i++) {
      // we need to check the randomization works. We have the shuffleing
      // and the amount executed, we need to make sure the balances
      // corresponding to the first executedCount shuffled indices
      // has one less unit of token
      const paraId = context.polkadotApi.createType("ParaId", i + 1) as any;
      const sovereignAddress = u8aToHex(
        new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
      ).padEnd(42, "0");

      const balance = await context.polkadotApi.query.system.account(sovereignAddress);

      if (
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.InitializationExecutedPassingBarrier ||
        shouldItExecute[indices.indexOf(i)] == XcmpExecution.OnIdleExecutedPassingBarrier
      ) {
        expect(balance.data.free.toBigInt()).to.eq(99n);
      } else {
        expect(balance.data.free.toBigInt()).to.eq(100n);
      }
    }
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test for two XCMP insertions for same para, the last is executed", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const instructionsNotExecuted = [
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
            fun: { Fungible: 1 },
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
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const instructionsExecuted = [
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
            fun: { Fungible: 2 },
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
            fun: { Fungible: 2 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const xcmMessageNotExecuted = {
      V2: instructionsNotExecuted,
    };
    const xcmMessageExecuted = {
      V2: instructionsExecuted,
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessageNotExecuted: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessageNotExecuted
    ) as any;

    const receivedMessageExecuted: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessageExecuted
    ) as any;

    const totalMessageNotExecuted = [...xcmpFormat.toU8a(), ...receivedMessageNotExecuted.toU8a()];
    const totalMessageExecuted = [...xcmpFormat.toU8a(), ...receivedMessageExecuted.toU8a()];

    const paraId = context.polkadotApi.createType("ParaId", 1) as any;
    const sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund each sovereign account with 100
    await context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n));

    // now we start injecting messages
    // two for para 1
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessageNotExecuted]);
    await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessageExecuted]);

    await context.createBlock();

    // The balance of the sovereign account should be 98, as the first message does not executed
    const balance = await context.polkadotApi.query.system.account(sovereignAddress);
    expect(balance.data.free.toBigInt()).to.eq(98n);
  });
});

describeDevMoonbeam("Mock XCMP - test XCMP execution", (context) => {
  it("Should test for three XCMP insertions, the channel gests suspended", async function () {
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;

    const instructionsNotExecuted = [
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
            fun: { Fungible: 1 },
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
            fun: { Fungible: 1 },
          },
          weightLimit: { Limited: new BN(20000000000) },
        },
      },
    ];

    const xcmMessageNotExecuted = {
      V2: instructionsNotExecuted,
    };

    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessageNotExecuted: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessageNotExecuted
    ) as any;

    const totalMessageNotExecuted = [...xcmpFormat.toU8a(), ...receivedMessageNotExecuted.toU8a()];

    const paraId = context.polkadotApi.createType("ParaId", 1) as any;
    const sovereignAddress = u8aToHex(
      new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
    ).padEnd(42, "0");

    // We first fund each sovereign account with 100
    // we will only withdraw 1, so no problem on this
    await context.createBlock(context.polkadotApi.tx.balances.transfer(sovereignAddress, 100n));

    const queueConfig = (await context.polkadotApi.query.xcmpQueue.queueConfig()) as any;
    const suspendThreshold = queueConfig.suspendThreshold.toNumber();

    // now we start injecting messages
    for (let i = 0; i < suspendThreshold + 1; i++) {
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessageNotExecuted]);
    }

    const result = await context.createBlock().catch((e) => e);
    expect(result.toString()).to.eq(
      "RpcError: 20000: Error at calling runtime api: " +
        "Execution failed: Runtime panicked: assertion failed: mid <= self.len()"
    );

    // The balance of the sovereign account should be 100, as none of the messages got executed
    const balance = await context.polkadotApi.query.system.account(sovereignAddress);
    expect(balance.data.free.toBigInt()).to.eq(100n);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM", (context) => {
  let transferredBalance;
  let DescendOriginAddress;
  let sendingAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;
    random = generateKeyingPair();
    const derivedMultiLocation = context.polkadotApi.createType(
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

    const toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
    );
    const balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const xcmTransaction = {
      V1: {
        gas_limit: 21000,
        fee_payment: {
          Auto: {
            Low: null,
          },
        },
        action: {
          Call: random.address,
        },
        value: transferredBalance / 10n,
        input: [],
        access_list: null,
      },
    };

    const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction);
    const transferCallEncoded = transferCall?.method.toHex();

    // We are going to test that we can receive a transact operation from parachain 1
    // using descendOrigin first
    const xcmMessage = {
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
            requireWeightAtMost: new BN(2000000000),
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
    const testAccountBalance = (
      await context.polkadotApi.query.system.account(random.address)
    ).data.free.toBigInt();

    expect(testAccountBalance).to.eq(transferredBalance / 10n);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM", (context) => {
  let transferredBalance;
  let DescendOriginAddress;
  let sendingAddress;
  let random: KeyringPair;
  let contractDeployed;

  before("Should receive transact action with DescendOrigin and deploy", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await context.createBlock(rawTx);

    contractDeployed = contract;

    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;
    random = generateKeyingPair();
    const derivedMultiLocation = context.polkadotApi.createType(
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

    const toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
    );
    const balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const xcmTransaction = {
      V1: {
        gas_limit: 100000,
        fee_payment: {
          Auto: {
            Low: null,
          },
        },
        action: {
          Call: contractDeployed.options.address,
        },
        value: 0n,
        input: contractDeployed.methods.incr().encodeABI().toString(),
        access_list: null,
      },
    };

    const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction);
    const transferCallEncoded = transferCall?.method.toHex();
    // We are going to test that we can receive a transact operation from parachain 1
    // using descendOrigin first
    const xcmMessage = {
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
            requireWeightAtMost: new BN(3000000000),
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

    expect(await contractDeployed.methods.count().call()).to.eq("1");
  });
});
