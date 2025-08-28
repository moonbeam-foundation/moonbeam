import "@moonbeam-network/api-augment";
import {
  beforeAll,
  describeSuite,
  expect,
  customDevRpcRequest,
  execOpenTechCommitteeProposal,
} from "@moonwall/cli";
import { ALITH_ADDRESS, alith, baltathar } from "@moonwall/util";
import type { u128 } from "@polkadot/types-codec";
import {
  RELAY_SOURCE_LOCATION,
  addAssetToWeightTrader,
  registerForeignAsset,
  mockAssetBalance,
  relayAssetMetadata,
  foreignAssetBalance,
} from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

describeSuite({
  id: "D022002",
  title: "Maintenance Mode - Filter2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    const foreignParaId = 2000;
    let foreignAssetId: u128;

    beforeAll(async () => {
      // registering asset using new foreign assets system
      const balance = 100000000000000n;

      // Register foreign asset
      const { registeredAssetId } = await registerForeignAsset(
        context,
        ARBITRARY_ASSET_ID,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata
      );

      assetId = context.polkadotJs().createType("u128", registeredAssetId);

      // Mock asset balance for baltathar
      await mockAssetBalance(
        context,
        balance,
        ARBITRARY_ASSET_ID,
        alith,
        baltathar.address as `0x{string}`
      );

      // set relative price in xcmWeightTrader
      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 0n, context);

      await execOpenTechCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it({
      id: "T01",
      title: "should queue DMP until resuming operations",
      test: async () => {
        // Send RPC call to inject DMP message
        // You can provide a message, but if you don't a downward transfer is the default
        await customDevRpcRequest("xcm_injectDownwardMessage", [[]]);

        // Create a block in which the XCM should be executed
        await context.createBlock();

        // Make sure the state does not have ALITH's DOT tokens
        const alithBalance = await foreignAssetBalance(context, ARBITRARY_ASSET_ID, ALITH_ADDRESS);

        // Alith balance is 0
        expect(alithBalance).to.eq(0n);

        // turn maintenance off
        await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
        );

        // Create a block in which the XCM will be executed
        await context.createBlock();

        // Make sure the state has ALITH's to DOT tokens
        const newAlithBalance = await foreignAssetBalance(
          context,
          ARBITRARY_ASSET_ID,
          ALITH_ADDRESS
        );

        // Alith balance is 10 DOT
        expect(newAlithBalance).to.eq(10000000000000n);
      },
    });
  },
});
