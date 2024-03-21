import "@moonbeam-network/api-augment";
import {
  beforeAll,
  beforeEach,
  customDevRpcRequest,
  describeSuite,
  expect,
  execOpenTechCommitteeProposal,
} from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { BN } from "@polkadot/util";

describeSuite({
  id: "D012103",
  title: "Maintenance Mode - Filter2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    const foreignParaId = 2000;

    beforeAll(async () => {
      const assetMetadata = {
        name: "FOREIGN",
        symbol: "FOREIGN",
        decimals: new BN(12),
        isFroze: false,
      };

      const sourceLocation = {
        Xcm: { parents: 1, interior: { X1: { Parachain: foreignParaId } } },
      };

      // registerForeignAsset
      const { result } = await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context
              .polkadotJs()
              .tx.assetManager.registerForeignAsset(sourceLocation, assetMetadata, new BN(1), true)
          )
      );

      const events = result?.events.find(
        ({ event: { section } }) => section.toString() === "assetManager"
      );

      if (!events) {
        throw new Error("Events Not Found!");
      }

      assetId = events.event.data[0].toHex().replace(/,/g, "");

      // setAssetUnitsPerSecond
      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context.polkadotJs().tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0, 0)
          )
      );
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
        let alithBalance = (await context
          .polkadotJs()
          .query.assets.account(assetId, ALITH_ADDRESS)) as any;
        // Alith balance is 0
        expect(alithBalance.isNone).to.eq(true);

        // turn maintenance off
        await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
        );

        // Create a block in which the XCM will be executed
        await context.createBlock();

        // Make sure the state has ALITH's to foreign assets tokens
        alithBalance = (
          await context.polkadotJs().query.assets.account(assetId, ALITH_ADDRESS)
        ).unwrap();

        expect(alithBalance.balance.toBigInt()).to.eq(10000000000000n);
      },
    });
  },
});
