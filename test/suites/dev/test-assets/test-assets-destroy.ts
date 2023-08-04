import "@moonbeam-network/api-augment";
import { u128 } from "@polkadot/types";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, baltathar } from "@moonwall/util";
import { mockAssetBalance } from "../../../helpers/assets.js";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { expectOk } from "../../../helpers/expect.js";
import { ApiPromise } from "@polkadot/api";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

describeSuite({
  id: "D0101",
  title: "Pallet Assets - Destruction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let api: ApiPromise;
    beforeAll(async () => {
      api = context.polkadotJs();
      assetId = api.createType("u128", ARBITRARY_ASSET_ID);
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = api.createType("Balance", 100000000000000);
      const assetBalance: PalletAssetsAssetAccount = api.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      const assetDetails: PalletAssetsAssetDetails = api.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      await mockAssetBalance(context, assetBalance, assetDetails, alith, assetId, ALITH_ADDRESS);
      await context.createBlock(api.tx.assets.transfer(assetId, baltathar.address, 1000));
    });

    it({
      id: "T01",
      title: "should destroy asset Balance",
      test: async function () {
        const metadataBefore = await context.polkadotJs().query.assets.metadata(assetId.toU8a());

        // Name is equal to "DOT" in hex
        expect(metadataBefore.name.toString()).to.eq("0x444f54");

        // assetDetails before in non-empty
        const assetDetailsBefore = await api.query.assets.asset(assetId.toU8a());
        expect(assetDetailsBefore.isNone).to.eq(false);

        // Destroy asset
        await expectOk(context.createBlock(api.tx.sudo.sudo(api.tx.assets.startDestroy(assetId))));
        await expectOk(context.createBlock(api.tx.assets.destroyAccounts(assetId)));
        await expectOk(context.createBlock(api.tx.assets.destroyApprovals(assetId)));
        await expectOk(context.createBlock(api.tx.assets.finishDestroy(assetId)));

        // Baltathar balance is None
        const baltatharBalance = await api.query.assets.account(assetId.toU8a(), BALTATHAR_ADDRESS);
        expect(baltatharBalance.isNone).to.eq(true);

        // metadata is default
        const metadata = await api.query.assets.metadata(assetId.toU8a());
        expect(metadata.name.toString()).to.eq("0x");

        // assetDetails is None
        const assetDetails = await api.query.assets.asset(assetId.toU8a());
        expect(assetDetails.isNone).to.eq(true);
      },
    });
  },
});
