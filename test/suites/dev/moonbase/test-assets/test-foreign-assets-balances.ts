import "@moonbeam-network/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import {
  ARBITRARY_ASSET_ID,
  RELAY_SOURCE_LOCATION_V4,
  foreignAssetBalance,
  mockAssetBalance,
} from "../../../../helpers";

describeSuite({
  id: "D010109",
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

        await mockAssetBalance(context, someBalance, assetId, assetLocation, alith, ALITH_ADDRESS);

        const newBalance = await foreignAssetBalance(context, assetId, ALITH_ADDRESS);
        expect(newBalance).toBe(someBalance);
      },
    });
  },
});
