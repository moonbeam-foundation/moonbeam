import "@moonbeam-network/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import { ALITH_ADDRESS, alith, describeSuite, expect } from "moonwall";
import {
  ARBITRARY_ASSET_ID,
  RELAY_SOURCE_LOCATION_V4,
  foreignAssetBalance,
  mockAssetBalance,
  registerForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";

describeSuite({
  id: "D020101",
  title: "XCM - Create new foreign asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should check balances consstency",
      test: async function () {
        const someBalance = 100_000_000_000_000n;
        const assetLocation = RELAY_SOURCE_LOCATION_V4;
        const assetId = ARBITRARY_ASSET_ID;

        // Register the asset
        await registerForeignAsset(context, assetId, assetLocation, relayAssetMetadata);
        // Mock asset balance
        await mockAssetBalance(context, someBalance, assetId, alith, ALITH_ADDRESS);

        const newBalance = await foreignAssetBalance(context, assetId, ALITH_ADDRESS);
        expect(newBalance).toBe(someBalance);
      },
    });
  },
});
