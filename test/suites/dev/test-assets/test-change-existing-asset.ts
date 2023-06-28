import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { BN, bnToHex } from "@polkadot/util";
import {
  PARA_1000_SOURCE_LOCATION,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
} from "../../../helpers/assets.js";
import { registerForeignAsset } from "../../../helpers/xcm.js";
import { verifyLatestBlockFees } from "../../../helpers/block.js";
import { ApiPromise } from "@polkadot/api";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
describeSuite({
  id: "D0106",
  title: "XCM - asset manager - Change existing asset",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: string;
    let api: ApiPromise;
    beforeAll(async function () {
      api = context.polkadotJs();
      const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata,
        1
      );
      assetId = registeredAssetId;
      expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
      expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
      expect(registeredAsset.owner.toString()).to.eq(palletId);

      await verifyLatestBlockFees(context);
    });

    it({
      id: "T01",
      title: "should change the asset Id",
      test: async function () {
        // ChangeAssetType
        await context.createBlock(
          api.tx.sudo.sudo(
            api.tx.assetManager.changeExistingAssetType(assetId, PARA_1000_SOURCE_LOCATION, 1)
          )
        );

        // asset_type
        const assetType = (await context
          .polkadotJs()
          .query.assetManager.assetIdType(assetId)) as Object;

        // assetId
        const id = (await api.query.assetManager.assetTypeId(PARA_1000_SOURCE_LOCATION)).unwrap();

        // asset units per second changed
        const assetUnitsPerSecond = (
          await api.query.assetManager.assetTypeUnitsPerSecond(PARA_1000_SOURCE_LOCATION)
        ).unwrap();

        // Supported assets
        const supportedAssets = await api.query.assetManager.supportedFeePaymentAssets();

        expect(assetUnitsPerSecond.toString()).to.eq(new BN(1).toString());
        expect(assetType.toString()).to.eq(JSON.stringify(PARA_1000_SOURCE_LOCATION).toLowerCase());
        expect(bnToHex(id)).to.eq(assetId);
        expect(supportedAssets[0].toString()).to.eq(
          JSON.stringify(PARA_1000_SOURCE_LOCATION).toLowerCase()
        );
      },
    });
  },
});
