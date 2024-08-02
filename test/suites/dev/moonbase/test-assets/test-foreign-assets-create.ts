import "@moonbeam-network/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import {
  ARBITRARY_ASSET_ID,
  RELAY_SOURCE_LOCATION_V4,
  registerForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";
import { parseAbi } from "viem";

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
          RELAY_SOURCE_LOCATION_V4,
          relayAssetMetadata as any
        );

      address = contractAddress;
      assetId = registeredAssetId;
      expect(contractAddress).toBeDefined();
      expect(registeredAssetId).eq(ARBITRARY_ASSET_ID.toString());
      expect(registeredAssetLocation.toString()).to.eq(
        JSON.stringify(RELAY_SOURCE_LOCATION_V4).toLowerCase()
      );
    });

    it({
      id: "T01",
      title: "should deploy the asset's contract",
      test: async function () {
        expect(
          await context.viem().readContract({
            address: address as `0x${string}`,
            functionName: "name",
            args: [],
            abi: parseAbi(["function name() view returns (string)"]),
          })
        ).toBe("DOT");

        expect(
          await context.viem().readContract({
            address: address as `0x${string}`,
            functionName: "symbol",
            args: [],
            abi: parseAbi(["function symbol() view returns (string)"]),
          })
        ).toBe("DOT");

        expect(
          await context.viem().readContract({
            address: address as `0x${string}`,
            functionName: "decimals",
            args: [],
            abi: parseAbi(["function decimals() view returns (uint8)"]),
          })
        ).toBe(12);
      },
    });
  },
});
