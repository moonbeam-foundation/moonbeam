import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
  registerForeignAsset,
} from "../../../../helpers";

describeSuite({
  id: "D010108",
  title: "XCM - Create new foreign asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should deploy the asset's contract",
      test: async function () {
        const { registeredAssetId, contractAddress, events } = await registerForeignAsset(
          context,
          RELAY_SOURCE_LOCATION,
          relayAssetMetadata as any
        );

        expect(contractAddress).toBeDefined();
        expect(registeredAssetId).eq("1");

        expect(
          await context.readContract!({
            contractName: "MyToken",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "name",
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
