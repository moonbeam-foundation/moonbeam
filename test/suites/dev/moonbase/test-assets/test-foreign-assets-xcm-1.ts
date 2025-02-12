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
  id: "D014110",
  title: "Create & manage Foreign Assets via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const assetId = 1;
    

    beforeAll(async () => {
      // Sibling Paras
      const siblingParas = [1000, 3333];
      const siblingParaSovereignAccounts = siblingParas.map((paraId) =>
        sovereignAccountOfSibling(context, paraId)
      );

      // Fund all accounts
      const fundAmount = 100_000_000_000_000_000_000_000n;
      for (const address of siblingParaSovereignAccounts) {
        await fundAccount(address as `0x${string}`, fundAmount, context);
      }
    });

    it({
      id: "T01",
      title: "SiblingPara should be able to create and manage a foreign asset via XCM",
      test: async function () {

        const assetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 1000 }, { PalletInstance: 1 }, { GeneralIndex: 1 }],
          },
        };

        const createForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, assetLocation, 18, "TEST", "TEST");

        const block = await sendCallAsPara(createForeignAssetCall, 1000, fundAmount / 20n, context);
    
        await expectEvent(context, block.hash as `0x${string}`, "ForeignAssetCreated");

        const createdForeignAsset = (
          await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
        ).toJSON();
        expect(createdForeignAsset).to.exist;
        expect(createdForeignAsset!["parents"]).to.eq(1);
        expect(createdForeignAsset!["interior"]["x3"][0]["parachain"]).to.eq(1000);
        expect(createdForeignAsset!["interior"]["x3"][1]["palletInstance"]).to.eq(1);
        expect(createdForeignAsset!["interior"]["x3"][2]["generalIndex"]).to.eq(1);

        const freezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);

        const block2 = await sendCallAsPara(freezeCall, 1000, fundAmount / 20n, context);
        await expectEvent(context, block2.hash as `0x${string}`, "ForeignAssetFrozen");

        const unfreezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(assetId);

        const block3 = await sendCallAsPara(unfreezeCall, 1000, fundAmount / 20n, context);
        await expectEvent(context, block3.hash as `0x${string}`, "ForeignAssetUnfrozen");

        const newAssetLocation = {
          parents: 1,
          interior: {
            X3: [{ Parachain: 1000 }, { PalletInstance: 2 }, { GeneralIndex: 2 }],
          },
        };

        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, newAssetLocation);
        
        const block4 = await sendCallAsPara(changeLocationCall, 1000, fundAmount / 20n, context);
        await expectEvent(context, block4.hash as `0x${string}`, "ForeignAssetXcmLocationChanged");

        const modifiedForeignAsset = (
          await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
        ).toJSON();
        expect(modifiedForeignAsset).to.exist;
        expect(modifiedForeignAsset!["parents"]).to.eq(1);
        expect(modifiedForeignAsset!["interior"]["x3"][0]["parachain"]).to.eq(1000);
        expect(modifiedForeignAsset!["interior"]["x3"][1]["palletInstance"]).to.eq(2);
        expect(modifiedForeignAsset!["interior"]["x3"][2]["generalIndex"]).to.eq(2);
      },
    });
  },
});

async function expectEvent(context: DevModeContext, blockHash: `0x${string}`, eventName: string) {
  const apiAt = await context.polkadotJs().at(blockHash);
  const events = await apiAt.query.system.events();
  const event = events.find(({ event: { method } }) => method.toString() === eventName)!.event;
  expect(event).to.exist;
  return event;
}

const getPalletIndex = async (name: string, context: DevModeContext) => {
  const metadata = await context.polkadotJs().rpc.state.getMetadata();
  return metadata.asLatest.pallets
    .find(({ name: palletName }) => palletName.toString() === name)!
    .index.toNumber();
};

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

const sendCallAsPara = async (
  call: any,
  paraId: number,
  fungible: bigint = 10_000_000_000_000_000_000n, // Default 10 GLMR
  context: DevModeContext
) => {
  const encodedCall = call.method.toHex();
  const balancesPalletIndex = await getPalletIndex("Balances", context);

  const xcmMessage = new XcmFragment({
    assets: [
      {
        multilocation: {
          parents: 0,
          interior: {
            X1: { PalletInstance: balancesPalletIndex },
          },
        },
        fungible: fungible
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
  const block = await injectHrmpMessageAndSeal(context, paraId, {
    type: "XcmVersionedXcm",
    payload: xcmMessage,
  } as RawXcmMessage);

  return block;
}
