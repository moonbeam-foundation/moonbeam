import "@moonbeam-network/api-augment";
import { afterEach, beforeAll, describeSuite, expect } from "@moonwall/cli";

import { sovereignAccountOfSibling, sendCallAsPara } from "../../../../helpers/xcm.js";
import { fundAccount } from "../../../../helpers/balances.js";
import type { AnyJson } from "@polkadot/types-codec/types";
import { expectSubstrateEvent } from "../../../../helpers/expect.js";

describeSuite({
  id: "D014113",
  title: "Changing a Foreign Asset XCM Location via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const fundAmount = 100_000_000_000_000_000_000_000n;
    const assetId = 4;
    let originalLocation: AnyJson;
    const firstAssetLocation = {
      parents: 1,
      interior: {
        x3: [{ Parachain: 5001 }, { palletInstance: 1 }, { generalIndex: 1 }],
      },
    };

    const secondAssetLocation = {
      parents: 1,
      interior: {
        x3: [{ Parachain: 5001 }, { palletInstance: 2 }, { generalIndex: 2 }],
      },
    };

    const anotherParaLocation = {
      parents: 1,
      interior: {
        x3: [{ Parachain: 5002 }, { palletInstance: 1 }, { generalIndex: 1 }],
      },
    };

    beforeAll(async () => {
      // Sibling Paras
      const siblingParas = [5001, 5002];
      const siblingParaSovereignAccounts = siblingParas.map((paraId) =>
        sovereignAccountOfSibling(context, paraId)
      );

      // Fund all accounts
      const fundAmount = 100_000_000_000_000_000_000_000n;
      for (const address of siblingParaSovereignAccounts) {
        await fundAccount(address as `0x${string}`, fundAmount, context);
      }

      // Create a foreign asset
      const createForeignAssetCall = context
        .polkadotJs()
        .tx.evmForeignAssets.createForeignAsset(assetId, firstAssetLocation, 18, "TEST", "TEST");
      const { blockRes } = await sendCallAsPara(
        createForeignAssetCall,
        5001,
        context,
        fundAmount / 20n
      );
      await expectSubstrateEvent(blockRes, "evmForeignAssets", "ForeignAssetCreated");

      originalLocation = (
        await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
      ).toJSON();
    });

    afterEach(async () => {
      // Reset asset state and expect it to be active
      const assetCurrentLocation = (
        await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
      ).toJSON();
      const assetByLocation = (
        await context.polkadotJs().query.evmForeignAssets.assetsByLocation(assetCurrentLocation)
      ).toJSON();
      if (assetByLocation![1] !== "Active") {
        const unfreezeForeignAssetCall = context
          .polkadotJs()
          .tx.evmForeignAssets.unfreezeForeignAsset(assetId);
        const sudoCall = context.polkadotJs().tx.sudo.sudo(unfreezeForeignAssetCall);
        await context.createBlock(sudoCall);
      }
      const assetAfter = (
        await context.polkadotJs().query.evmForeignAssets.assetsByLocation(assetCurrentLocation)
      ).toJSON();
      expect(assetAfter![1]).to.eq("Active");

      // Reset asset location
      if (assetCurrentLocation!.toString() !== originalLocation!.toString()) {
        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, firstAssetLocation);
        const sudoCall = context.polkadotJs().tx.sudo.sudo(changeLocationCall);
        const block = await context.createBlock(sudoCall);
        await expectSubstrateEvent(block, "evmForeignAssets", "ForeignAssetXcmLocationChanged");
      }
    });

    it({
      id: "T01",
      title: "Owner should be able to change XCM location in same Para and keep ownership",
      test: async function () {
        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, secondAssetLocation);
        const { blockRes: block1 } = await sendCallAsPara(
          changeLocationCall,
          5001,
          context,
          fundAmount / 20n
        );
        await expectSubstrateEvent(block1, "evmForeignAssets", "ForeignAssetXcmLocationChanged");

        const freezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);
        const { blockRes: block2 } = await sendCallAsPara(
          freezeCall,
          5001,
          context,
          fundAmount / 20n
        );
        await expectSubstrateEvent(block2, "evmForeignAssets", "ForeignAssetFrozen");
      },
    });

    it({
      id: "T02",
      title: "Non-owner should not be able to change XCM location",
      test: async function () {
        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, anotherParaLocation);
        const { errorName } = await sendCallAsPara(
          changeLocationCall,
          5002,
          context,
          fundAmount / 20n,
          true
        );
        expect(errorName).to.eq("LocationOutsideOfOrigin");
      },
    });

    it({
      id: "T03",
      title: "Owner should not be able to change XCM location to a different Para",
      test: async function () {
        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, anotherParaLocation);
        const { errorName } = await sendCallAsPara(
          changeLocationCall,
          5001,
          context,
          fundAmount / 20n,
          true
        );
        expect(errorName).to.eq("LocationOutsideOfOrigin");
      },
    });

    it({
      id: "T04",
      title: "Gov/Sudo should be able to change XCM location to different Para",
      test: async function () {
        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(assetId, anotherParaLocation);
        const sudoCall = context.polkadotJs().tx.sudo.sudo(changeLocationCall);
        const block1 = await context.createBlock(sudoCall);
        await expectSubstrateEvent(block1, "evmForeignAssets", "ForeignAssetXcmLocationChanged");

        // New para can manage the asset
        const freezeCall = context
          .polkadotJs()
          .tx.evmForeignAssets.freezeForeignAsset(assetId, false);
        const { blockRes: block2 } = await sendCallAsPara(
          freezeCall,
          5002,
          context,
          fundAmount / 20n
        );
        await expectSubstrateEvent(block2, "evmForeignAssets", "ForeignAssetFrozen");
      },
    });

    it({
      id: "T05",
      title: "Should not be able to change XCM location if asset does not exist",
      test: async function () {
        const changeLocationCall = context
          .polkadotJs()
          .tx.evmForeignAssets.changeXcmLocation(255, secondAssetLocation);
        const { errorName } = await sendCallAsPara(
          changeLocationCall,
          5001,
          context,
          fundAmount / 20n,
          true
        );
        expect(errorName).to.eq("AssetDoesNotExist");
      },
    });
  },
});
