import "@moonbeam-network/api-augment";
import {
  ALITH_ADDRESS,
  beforeAll,
  beforeEach,
  customDevRpcRequest,
  describeSuite,
  execOpenTechCommitteeProposal,
  expect,
} from "moonwall";
import { BN } from "@polkadot/util";
import {
  registerForeignAsset,
  addAssetToWeightTrader,
  PARA_2000_SOURCE_LOCATION,
  foreignAssetBalance,
} from "../../../../helpers";

describeSuite({
  id: "D021903",
  title: "Maintenance Mode - Filter2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    const foreignParaId = 2000;

    beforeAll(async () => {
      const assetMetadata = {
        name: "FOREIGN",
        symbol: "FOREIGN",
        decimals: 12n,
        isFrozen: false,
      };

      const arbitraryAssetId = 42259045809535163221576417993425387649n; // Different from other tests

      // Register foreign asset using the new system
      const { registeredAssetId } = await registerForeignAsset(
        context,
        arbitraryAssetId,
        PARA_2000_SOURCE_LOCATION,
        assetMetadata
      );

      assetId = registeredAssetId.toString();

      // set relative price in xcmWeightTrader
      await addAssetToWeightTrader(PARA_2000_SOURCE_LOCATION, 0n, context);
    });

    beforeEach(async () => {
      await execOpenTechCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it({
      id: "T01",
      title: "should queue XCM messages until resuming operations",
      test: async () => {
        // Send RPC call to inject XCMP message
        // You can provide a message, but if you don't a downward transfer is the default
        await customDevRpcRequest("xcm_injectHrmpMessage", [foreignParaId, []]);

        // Create a block in which the XCM should be executed
        await context.createBlock();

        // Make sure the state does not have ALITH's foreign asset tokens
        let alithBalance = await foreignAssetBalance(context, BigInt(assetId), ALITH_ADDRESS);
        // Alith balance is 0
        expect(alithBalance).to.eq(0n);

        // turn maintenance off
        await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
        );

        // Create a block in which the XCM will be executed
        await context.createBlock();

        // Make sure the state has ALITH's to foreign assets tokens
        alithBalance = await foreignAssetBalance(context, BigInt(assetId), ALITH_ADDRESS);

        expect(alithBalance).to.eq(10000000000000n);
      },
    });
  },
});
