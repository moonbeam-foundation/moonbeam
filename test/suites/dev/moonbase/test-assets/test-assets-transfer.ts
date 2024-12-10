import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, alith, baltathar } from "@moonwall/util";
import "@polkadot/api-augment";
import type { u128 } from "@polkadot/types";
import type { ApiPromise } from "@polkadot/api";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { mockOldAssetBalance } from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

describeSuite({
  id: "D010105",
  title: "Pallet Assets - Transfer",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
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

      await mockOldAssetBalance(context, assetBalance, assetDetails, alith, assetId, ALITH_ADDRESS);
    });

    it({
      id: "T01",
      title: "should be sucessfull",
      test: async function () {
        const { result } = await context.createBlock(
          api.tx.assets.transfer(assetId, baltathar.address, 1000)
        );

        expect(result!.error).to.be.undefined;

        // Baltathar balance is 1000
        const baltatharBalance = await api.query.assets.account(assetId.toU8a(), BALTATHAR_ADDRESS);
        expect(baltatharBalance.unwrap().balance.toBigInt()).to.equal(1000n);
      },
    });
  },
});
