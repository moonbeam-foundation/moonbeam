import "@moonbeam-network/api-augment";
import "@polkadot/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR } from "@moonwall/util";
import { BN } from "@polkadot/util";
import { verifyLatestBlockFees } from "../../../helpers/block.js";
import { expectOk } from "../../../helpers/expect.js";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D0108",
  title: "XCM - asset manager - destroy local asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;
    let api: ApiPromise;
    beforeAll(async function () {
      api = context.polkadotJs();
      // Check ALITH has amount reserved
      const accountDetailsBefore = await api.query.system.account(ALITH_ADDRESS);

      // registerAsset
      const { result } = await context.createBlock(
        api.tx.sudo.sudo(
          api.tx.assetManager.registerLocalAsset(ALITH_ADDRESS, ALITH_ADDRESS, true, new BN(1))
        )
      );

      assetId = result?.events
        .find(({ event: { section } }) => section.toString() === "assetManager")
        .event.data[0].toHex()
        .replace(/,/g, "");

      // check asset in storage
      const registeredAsset = (await api.query.localAssets.asset(assetId)).unwrap();
      expect(registeredAsset.owner.toString()).to.eq(ALITH_ADDRESS);

      // Check ALITH has amount reserved
      const accountDetails = await api.query.system.account(ALITH_ADDRESS);
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
        const accountDetailsBefore = await api.query.system.account(ALITH_ADDRESS);

        await expectOk(
          context.createBlock(api.tx.sudo.sudo(api.tx.assetManager.destroyLocalAsset(assetId)))
        );
        await expectOk(context.createBlock(api.tx.localAssets.destroyAccounts(assetId)));
        await expectOk(context.createBlock(api.tx.localAssets.destroyApprovals(assetId)));
        await expectOk(context.createBlock(api.tx.localAssets.finishDestroy(assetId)));

        // assetDetails should have dissapeared
        let assetDetails = await api.query.localAssets.asset(assetId);
        expect(assetDetails.isNone).to.eq(true);

        // Reserved amount back to creator
        const accountDetailsAfter = await api.query.system.account(ALITH_ADDRESS);

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
