import "@moonbeam-network/api-augment/moonbase";
import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  RELAY_SOURCE_LOCATION_V4,
  relayAssetMetadata,
  registerForeignAsset,
} from "../../../../helpers";
import { StagingXcmV4Location } from "@polkadot/types/lookup";

describeSuite({
  id: "D010108",
  title: "XCM - Create new foreign asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should deploy the asset's contract",
      test: async function () {
        const { registeredAssetId, contractAddress, registeredAssetLocation } =
          await registerForeignAsset(
            context,
            RELAY_SOURCE_LOCATION_V4.Xcm,
            relayAssetMetadata as any
          );

        expect(contractAddress).toBeDefined();
        expect(registeredAssetId).eq("1");
        expect(registeredAssetLocation.toString()).to.eq(
          JSON.stringify(RELAY_SOURCE_LOCATION_V4.Xcm).toLowerCase()
        );

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "symbol",
            args: [],
          })
        ).toBe("DOT");

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "symbol",
            args: [],
          })
        ).toBe("DOT");

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "decimals",
            args: [],
          })
        ).toBe(12);
      },
    });
  },
});
