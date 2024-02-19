import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { bnToHex } from "@polkadot/util";

import { ApiPromise } from "@polkadot/api";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D010109",
  title: "XCM - asset manager - Remove asset from supported",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: string;
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata as any,
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
      title: "should remove an asset from our supported fee payments",
      test: async function () {
        // ChangeAssetType
        await context.createBlock(
          api.tx.sudo.sudo(api.tx.assetManager.removeSupportedAsset(RELAY_SOURCE_LOCATION, 1))
        );

        // assetId
        const id = (await api.query.assetManager.assetTypeId(RELAY_SOURCE_LOCATION)).unwrap();

        // asset units per second removed
        const assetUnitsPerSecond = await api.query.assetManager.assetTypeUnitsPerSecond(
          RELAY_SOURCE_LOCATION
        );

        // Supported assets should be 0
        const supportedAssets = await api.query.assetManager.supportedFeePaymentAssets();

        expect(assetUnitsPerSecond.isNone).to.eq(true);
        expect(bnToHex(id)).to.eq(assetId);
        // the asset should not be supported
        expect(supportedAssets.length).to.eq(0);
      },
    });
  },
});
import {
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
  registerForeignAsset,
  verifyLatestBlockFees,
} from "../../../../helpers";
