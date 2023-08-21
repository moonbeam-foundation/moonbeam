// import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { BN } from "@polkadot/util";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

describeSuite({
  id: "D0110",
  title: "XCM - asset manager - register local asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to register a local asset",
      test: async function () {
        const parachainOne = context.polkadotJs();
        // registerForeignAsset
        const { result } = await context.createBlock(
          parachainOne.tx.sudo.sudo(
            parachainOne.tx.assetManager.registerLocalAsset(
              ALITH_ADDRESS,
              ALITH_ADDRESS,
              true,
              new BN(1)
            )
          )
        );
        // Look for assetId in events
        const assetId: string = result?.events
          .find(({ event: { section } }) => section.toString() === "assetManager")
          .event.data[0].toHex()
          .replace(/,/g, "");

        // check asset in storage
        const registeredAsset = (await parachainOne.query.localAssets.asset(assetId)).unwrap();
        expect(registeredAsset.owner.toString()).to.eq(ALITH_ADDRESS);

        // check deposit in storage
        const deposit = (await parachainOne.query.assetManager.localAssetDeposit(assetId)).unwrap();
        expect(deposit.creator.toString()).to.eq(ALITH_ADDRESS);

        await verifyLatestBlockFees(context);
      },
    });
  },
});
