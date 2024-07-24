import "@moonbeam-network/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import {
  ARBITRARY_ASSET_ID,
  RELAY_SOURCE_LOCATION_V4,
  foreignAssetBalance,
  mockAssetBalance,
  registerForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";

describeSuite({
  id: "D010108",
  title: "XCM - Create new foreign asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let address: string;
    let assetId: bigint;

    beforeAll(async () => {
      const { registeredAssetId, contractAddress, registeredAssetLocation } =
        await registerForeignAsset(
          context,
          ARBITRARY_ASSET_ID,
          RELAY_SOURCE_LOCATION_V4.Xcm,
          relayAssetMetadata as any
        );

      address = contractAddress;
      assetId = registeredAssetId;
      expect(contractAddress).toBeDefined();
      expect(registeredAssetId).eq(ARBITRARY_ASSET_ID.toString());
      expect(registeredAssetLocation.toString()).to.eq(
        JSON.stringify(RELAY_SOURCE_LOCATION_V4.Xcm).toLowerCase()
      );
    });

    it({
      id: "T01",
      title: "should deploy the asset's contract",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: address as `0x${string}`,
            functionName: "symbol",
            args: [],
          })
        ).toBe("DOT");

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: address as `0x${string}`,
            functionName: "symbol",
            args: [],
          })
        ).toBe("DOT");

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: address as `0x${string}`,
            functionName: "symbol",
            args: [],
          })
        ).toBe("DOT");

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: address as `0x${string}`,
            functionName: "decimals",
            args: [],
          })
        ).toBe(12);
      },
    });
  },
});
