import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import {
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";
import { expectEvent, sendCallAsPara } from "./test-foreign-assets-xcm-0-utils.js";

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


