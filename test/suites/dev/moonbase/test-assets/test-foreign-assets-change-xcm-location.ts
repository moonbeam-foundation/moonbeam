import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import {
  PARA_1000_SOURCE_LOCATION_V4,
  RELAY_SOURCE_LOCATION_V4,
  registerForeignAsset,
  relayAssetMetadata,
  verifyLatestBlockFees,
} from "../../../../helpers";

describeSuite({
  id: "D010106",
  title: "XCM - Change existing asset's XCM location",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: bigint;
    let api: ApiPromise;
    beforeAll(async function () {
      api = context.polkadotJs();
      const { registeredAssetId } = await registerForeignAsset(
        context,
        BigInt(1),
        RELAY_SOURCE_LOCATION_V4,
        relayAssetMetadata as any
      );
      assetId = registeredAssetId;

      await verifyLatestBlockFees(context);
    });

    it({
      id: "T01",
      title: "should change the asset location",
      test: async function () {
        const { result } = await context.createBlock(
          api.tx.sudo.sudo(
            api.tx.evmForeignAssets.changeXcmLocation(assetId, PARA_1000_SOURCE_LOCATION_V4)
          )
        );

        const locationChangeEvent = (result as any).events.find(
          ({ event: { method } }) => method.toString() === "ForeignAssetXcmLocationChanged"
        ).event;

        const newLocation = locationChangeEvent.data[1];
        const id = locationChangeEvent.data[0];

        expect(JSON.stringify(newLocation).toLowerCase()).to.eq(
          JSON.stringify(PARA_1000_SOURCE_LOCATION_V4).toLowerCase()
        );
        expect(BigInt(id)).to.eq(BigInt(assetId));
      },
    });
  },
});
