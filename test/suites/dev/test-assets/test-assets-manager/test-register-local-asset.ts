import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { BN } from "@polkadot/util";
import { verifyLatestBlockFees } from "../../../../helpers/block.js";

describeSuite({
  id: "D115",
  title: "XCM - asset manager - register local asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to register a local asset",
      test: async function () {
        const parachainOne = context.polkadotJs();
        // registerForeignAsset
        const {
          result: { events: eventsRegister },
        } = await context.createBlock(
          parachainOne.tx.sudo.sudo(
            parachainOne.tx.assetManager.registerLocalAsset(
              alith.address,
              alith.address,
              true,
              new BN(1)
            )
          )
        );
        // Look for assetId in events
        const assetId = eventsRegister
          .find(({ event: { section } }) => section.toString() === "assetManager")
          .event.data[0].toHex()
          .replace(/,/g, "");

        // check asset in storage
        const registeredAsset = (await parachainOne.query.localAssets.asset(assetId)).unwrap();
        expect(registeredAsset.owner.toString()).to.eq(alith.address);

        // check deposit in storage
        const deposit = (await parachainOne.query.assetManager.localAssetDeposit(assetId)).unwrap();
        expect(deposit.creator.toString()).to.eq(alith.address);

        await verifyLatestBlockFees(context);
      },
    });
  },
});
