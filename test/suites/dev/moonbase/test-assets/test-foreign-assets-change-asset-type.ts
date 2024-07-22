import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { BN, bnToHex } from "@polkadot/util";
import {
  PARA_1000_SOURCE_LOCATION,
  RELAY_SOURCE_LOCATION,
  registerForeignAsset,
  relayAssetMetadata,
  verifyLatestBlockFees,
} from "../../../../helpers";
import { u128 } from "@polkadot/types-codec";
import { StagingXcmV4Location } from "@polkadot/types/lookup";

describeSuite({
  id: "D010106",
  title: "XCM - asset manager - Change existing asset",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: u128;
    let api: ApiPromise;
    beforeAll(async function () {
      api = context.polkadotJs();
      const { registeredAssetId, events } = await registerForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata as any
      );
      assetId = registeredAssetId;

      await verifyLatestBlockFees(context);
    });

    it({
      id: "T01",
      title: "should change the asset type",
      test: async function () {
        // ChangeAssetType
        const { result } = await context.createBlock(
          api.tx.sudo.sudo(
            api.tx.evmForeignAssets.changeExistingAssetType(assetId, PARA_1000_SOURCE_LOCATION)
          )
        );

        const assetTypeChangedEvent = (result as any).events.find(
          ({ event: { method } }) => method.toString() === "ForeignAssetTypeChanged"
        )!.event;

        // asset_type
        const assetType = assetTypeChangedEvent.data[1];

        // assetId
        const id: u128 = assetTypeChangedEvent.data[0];

        expect(assetType.toString()).to.eq(JSON.stringify(PARA_1000_SOURCE_LOCATION).toLowerCase());
        expect(id).to.eq(assetId);
      },
    });
  },
});
