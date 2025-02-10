import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, type DevModeContext, expect } from "@moonwall/cli";

import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";

describeSuite({
  id: "D014199",
  title: "Create Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    let descendAddress: `0x${string}`;

    beforeAll(async () => {
      // Derive Sovereign Account for the sibling parachain
      const siblingSovereignAccount = sovereignAccountOfSibling(context, 1000) as `0x${string}`;
      await fundAccount(siblingSovereignAccount, fundAmount, context);

      const randomForeignAccount = generateKeyringPair();
      const { descendOriginAddress } = descendOriginFromAddress20(
        context,
        randomForeignAccount.address as `0x${string}`,
        1000
      );
      descendAddress = descendOriginAddress;
      await fundAccount(descendAddress, fundAmount, context);
    });

    it({
      id: "T03",
      title: "SiblingPara should be able to create a foreign asset via XCM",
      test: async function () {
        const balancesPalletIndex = await getPalletIndex("Balances", context);

        const assetId = 2;
        const assetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 1000 }, { PalletInstance: 1 }, { GeneralIndex: 0 }],
          },
        };
        const assetCreationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const encodedCall = assetCreationCall?.method.toHex();

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: fundAmount / 20n,
            },
          ],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 120_000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "Xcm",
              requireWeightAtMost: {
                refTime: 20_089_165_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v4();

        // Send an XCM and create block to execute it
        const block = await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetCreated");

        const createdForeignAsset = (
          await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
        ).toJSON();
        expect(createdForeignAsset).to.exist;
        expect(createdForeignAsset!["parents"]).to.eq(1);
        expect(createdForeignAsset!["interior"]["x3"][0]["parachain"]).to.eq(1000);
        expect(createdForeignAsset!["interior"]["x3"][1]["palletInstance"]).to.eq(1);
        expect(createdForeignAsset!["interior"]["x3"][2]["generalIndex"]).to.eq(0);
      },
    });

    it({
      id: "T04",
      title: "SiblingPara should be able to freeze a foreign asset via XCM",
      test: async function () {
        const balancesPalletIndex = await getPalletIndex("Balances", context);

        const assetId = 2;
        const assetFreezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);

        const encodedCall = assetFreezeCall?.method.toHex();

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: fundAmount / 20n,
            },
          ],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 120_000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "Xcm",
              requireWeightAtMost: {
                refTime: 20_089_165_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v4();

        // Send an XCM and create block to execute it
        const block = await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetFrozen");
      },
    });

    it({
      id: "T05",
      title: "SiblingPara should be able to unfreeze a foreign asset via XCM",
      test: async function () {
        const balancesPalletIndex = await getPalletIndex("Balances", context);

        const assetId = 2;

        const assetUnfreezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(assetId);

        const encodedCall = assetUnfreezeCall?.method.toHex();

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: fundAmount / 20n,
            },
          ],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 120_000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "Xcm",
              requireWeightAtMost: {
                refTime: 20_089_165_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v4();

        // Send an XCM and create block to execute it
        const block = await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetUnfrozen");
      },
    });

    it({
      id: "T06",
      title:
        "SiblingPara should be able to change XCM location of a foreign asset located in itself",
      test: async function () {
        const balancesPalletIndex = await getPalletIndex("Balances", context);

        const assetId = 2;
        const newAssetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 1000 }, { PalletInstance: 2 }, { GeneralIndex: 1 }],
          },
        };

        const assetChangeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, newAssetLocation);

        const encodedCall = assetChangeLocationCall?.method.toHex();

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: fundAmount / 20n,
            },
          ],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 120_000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "Xcm",
              requireWeightAtMost: {
                refTime: 20_089_165_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v4();

        // Send an XCM and create block to execute it
        const block = await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetXcmLocationChanged");
      },
    });

    it({
      id: "T07",
      title: "SiblingPara should NOT be able to change location if not owning current",
      test: async function () {
        const balancesPalletIndex = await getPalletIndex("Balances", context);

        const assetId = 2;

        const before = await getForeignAssetDetails(assetId, context);
        const newAssetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 1000 }, { PalletInstance: 2 }, { GeneralIndex: 1 }],
          },
        };

        const assetChangeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, newAssetLocation);

        const encodedCall = assetChangeLocationCall?.method.toHex();

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: fundAmount / 20n,
            },
          ],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 120_000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "Xcm",
              requireWeightAtMost: {
                refTime: 20_089_165_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v4();

        // Send an XCM from different para and create block to execute it
        const block = await injectHrmpMessageAndSeal(context, 3333, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const after = await getForeignAssetDetails(assetId, context);
        expect(after).to.toStrictEqual(before);
      },
    });

    it({
      id: "T07",
      title: "SiblingPara should NOT be able to change location if not owning new one",
      test: async function () {
        const balancesPalletIndex = await getPalletIndex("Balances", context);

        const assetId = 2;

        const before = await getForeignAssetDetails(assetId, context);
        const newAssetLocation = {
          parents: 1,
          interior: {
            // Change it to different Para
            X3: [{ Parachain: 3333 }, { PalletInstance: 2 }, { GeneralIndex: 1 }],
          },
        };

        const assetChangeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, newAssetLocation);

        const encodedCall = assetChangeLocationCall?.method.toHex();

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: fundAmount / 20n,
            },
          ],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 120_000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "Xcm",
              requireWeightAtMost: {
                refTime: 20_089_165_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v4();

        // Send an XCM from different para and create block to execute it
        const block = await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const after = await getForeignAssetDetails(assetId, context);
        expect(after).toStrictEqual(before);
      },
    });
  },
});

async function expectEvent(context: DevModeContext, blockHash: `0x${string}`, eventName: string) {
  const apiAt = await context.polkadotJs().at(blockHash);
  const events = await apiAt.query.system.events();
  const event = events.find(({ event: { method } }) => method.toString() === eventName)!.event;
  expect(event).to.exist;
}

const getPalletIndex = async (name: string, context: DevModeContext) => {
  const metadata = await context.polkadotJs().rpc.state.getMetadata();
  return metadata.asLatest.pallets
    .find(({ name: palletName }) => palletName.toString() === name)!
    .index.toNumber();
};

interface ForeignAssetDetails {
  parents: number;
  interior: {
    X3: [{ Parachain: number }, { PalletInstance: number }, { GeneralIndex: number }];
  };
}

const getForeignAssetDetails = async (assetId: number, context: DevModeContext) => {
  const createdForeignAsset = (
    await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
  ).toJSON();
  const assetDetails = {
    parents: createdForeignAsset!["parents"],
    interior: {
      X3: [
        { Parachain: createdForeignAsset!["interior"]["x3"][0]["parachain"] },
        { PalletInstance: createdForeignAsset!["interior"]["x3"][1]["palletInstance"] },
        { GeneralIndex: createdForeignAsset!["interior"]["x3"][2]["generalIndex"] },
      ],
    },
  };
  return assetDetails;
};

const expectForeignAssetDetails = async (
  assetId: number,
  expectedDetails: ForeignAssetDetails,
  context: DevModeContext
) => {
  const createdForeignAsset = await getForeignAssetDetails(assetId, context);
  expect(createdForeignAsset).to.exist;
  expect(createdForeignAsset).to.eq(expectedDetails);
};
