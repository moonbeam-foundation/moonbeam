import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { RELAY_SOURCE_LOCATION2, relayAssetMetadata } from "../../../helpers/assets.js";
import { registerForeignAsset } from "../../../helpers/xcm.js";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

describeSuite({
  id: "D0109",
  title: "XCM - asset manager - foreign asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be registerable and have unit per seconds set",
      test: async function () {
        const { events, registeredAsset } = await registerForeignAsset(
          context,
          RELAY_SOURCE_LOCATION2,
          relayAssetMetadata
        );

        expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
        expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
        expect(registeredAsset.owner.toString()).to.eq(palletId);
        await verifyLatestBlockFees(context);
      },
    });
  },
});
