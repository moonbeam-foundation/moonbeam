import "@moonbeam-network/api-augment";
import "@polkadot/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { alith, GLMR } from "@moonwall/util";
import { BN } from "@polkadot/util";
import { verifyLatestBlockFees } from "../../../../helpers/block.js";
import { expectOk } from "../../../../helpers/expect.js";

describeSuite({
  id: "AM6",
  title: "XCM - asset manager - destroy local asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    let api;
    beforeAll(async function () {
      api = context.polkadotJs();
      // Check ALITH has amount reserved
      const accountDetailsBefore: any = await api.query.system.account(alith.address);

      // registerAsset
      const {
        result: { events: eventsRegister },
      } = await context.createBlock(
        api.tx.sudo.sudo(
          api.tx.assetManager.registerLocalAsset(alith.address, alith.address, true, new BN(1))
        )
      );

      assetId = eventsRegister
        .find(({ event: { section } }) => section.toString() === "assetManager")
        .event.data[0].toHex()
        .replace(/,/g, "");

      // check asset in storage
      const registeredAsset = (await api.query.localAssets.asset(assetId)).unwrap();
      expect(registeredAsset.owner.toString()).to.eq(alith.address);

      // Check ALITH has amount reserved
      const accountDetails: any = await api.query.system.account(alith.address);
      expect(accountDetails.data.reserved.toString()).to.eq(
        (accountDetailsBefore.data.reserved.toBigInt() + 100n * GLMR).toString()
      );
      await verifyLatestBlockFees(context);
    });

    it({
      id: "T01",
      title: "should be able to destroy a local asset through pallet-asset-manager",
      test: async function () {
        // Reserved amount back to creator
        const accountDetailsBefore: any = await api.query.system.account(alith.address);

        await expectOk(
          context.createBlock(
            api.tx.sudo.sudo((api.tx.assetManager as any).destroyLocalAsset(assetId))
          )
        );
        await expectOk(context.createBlock(api.tx.localAssets.destroyAccounts(assetId)));
        await expectOk(context.createBlock(api.tx.localAssets.destroyApprovals(assetId)));
        await expectOk(context.createBlock(api.tx.localAssets.finishDestroy(assetId)));

        // assetDetails should have dissapeared
        let assetDetails = await api.query.localAssets.asset(assetId);
        expect(assetDetails.isNone).to.eq(true);

        // Reserved amount back to creator
        const accountDetailsAfter: any = await api.query.system.account(alith.address);

        // Amount should have decreased in one GLMR
        expect(accountDetailsAfter.data.reserved.toString()).to.eq(
          (accountDetailsBefore.data.reserved.toBigInt() - 100n * GLMR).toString()
        );

        // check deposit not in storage
        const deposit = await api.query.assetManager.localAssetDeposit(assetId);
        expect(deposit.isNone).to.eq(true);
      },
    });
  },
});
