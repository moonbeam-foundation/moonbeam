import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { RELAY_SOURCE_LOCATION, relayAssetMetadata } from "../../../helpers/assets.js";
import { registerForeignAsset } from "../../../helpers/xcm.js";
import { verifyLatestBlockFees } from "../../../helpers/block.js";
import { expectOk } from "../../../helpers/expect.js";
import { ApiPromise } from "@polkadot/api";
const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D0107",
  title: "XCM - asset manager - destroy foreign asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    let api: ApiPromise;
    beforeAll(async function () {
      api = context.polkadotJs();
      // registerForeignAsset
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
      title: "should be able to destroy a foreign asset through pallet-asset-manager",
      test: async function () {
        // Destroy foreign asset
        await expectOk(
          context.createBlock(
            api.tx.sudo.sudo((api.tx.assetManager as any).destroyForeignAsset(assetId, 1))
          )
        );

        await expectOk(context.createBlock(api.tx.assets.destroyAccounts(assetId)));
        await expectOk(context.createBlock(api.tx.assets.destroyApprovals(assetId)));
        await expectOk(context.createBlock(api.tx.assets.finishDestroy(assetId)));

        // assetId
        const id = await api.query.assetManager.assetTypeId(RELAY_SOURCE_LOCATION);

        // asset units per second removed
        const assetUnitsPerSecond = await api.query.assetManager.assetTypeUnitsPerSecond(
          RELAY_SOURCE_LOCATION
        );

        // Supported assets should be 0
        const supportedAssets = await api.query.assetManager.supportedFeePaymentAssets();

        // assetDetails should have dissapeared
        const assetDetails = await api.query.assets.asset(assetId);

        expect(assetUnitsPerSecond.isNone).to.eq(true);
        expect(id.isNone).to.eq(true);
        expect(assetDetails.isNone).to.eq(true);
        // the asset should not be supported
        expect(supportedAssets.length).to.eq(0);
      },
    });
  },
});
