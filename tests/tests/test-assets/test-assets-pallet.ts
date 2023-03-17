import "@moonbeam-network/api-augment";

import { u128 } from "@polkadot/types";
import { expect } from "chai";

import { alith, baltathar } from "../../util/accounts";
import { mockAssetBalance } from "../../util/assets";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { expectOk } from "../../util/expect";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

describeDevMoonbeam("Pallet Assets - Transfer", (context) => {
  let assetId: u128;
  before("Setup asset", async () => {
    assetId = context.polkadotApi.createType("u128", ARBITRARY_ASSET_ID);
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
      "PalletAssetsAssetAccount",
      {
        balance: balance,
      }
    );

    const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
      "PalletAssetsAssetDetails",
      {
        supply: balance,
      }
    );

    await mockAssetBalance(context, assetBalance, assetDetails, alith, assetId, alith.address);
  });

  it("should be sucessfull", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.assets.transfer(assetId, baltathar.address, 1000)
    );

    expect(result.error).to.be.undefined;

    // Baltathar balance is 1000
    const baltatharBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      baltathar.address
    );
    expect(baltatharBalance.unwrap().balance.toBigInt()).to.equal(1000n);
  });
});

describeDevMoonbeam("Pallet Assets - Destruction", (context) => {
  let assetId: u128;
  before("Setup asset & balance", async () => {
    assetId = context.polkadotApi.createType("u128", ARBITRARY_ASSET_ID);
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
      "PalletAssetsAssetAccount",
      {
        balance: balance,
      }
    );

    const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
      "PalletAssetsAssetDetails",
      {
        supply: balance,
      }
    );

    await mockAssetBalance(context, assetBalance, assetDetails, alith, assetId, alith.address);

    await context.createBlock(
      context.polkadotApi.tx.assets.transfer(assetId, baltathar.address, 1000)
    );
  });

  it("should destroy asset Balance, ", async function () {
    const metadataBefore = await context.polkadotApi.query.assets.metadata(assetId.toU8a());

    // Name is equal to "DOT" in hex
    expect(metadataBefore.name.toString()).to.eq("0x444f54");

    // assetDetails before in non-empty
    const assetDetailsBefore = await context.polkadotApi.query.assets.asset(assetId.toU8a());
    expect(assetDetailsBefore.isNone).to.eq(false);

    // Destroy asset
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(context.polkadotApi.tx.assets.startDestroy(assetId))
      )
    );
    await expectOk(context.createBlock(context.polkadotApi.tx.assets.destroyAccounts(assetId)));
    await expectOk(context.createBlock(context.polkadotApi.tx.assets.destroyApprovals(assetId)));
    await expectOk(context.createBlock(context.polkadotApi.tx.assets.finishDestroy(assetId)));

    // Baltathar balance is None
    const baltatharBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      baltathar.address
    );
    expect(baltatharBalance.isNone).to.eq(true);

    // metadata is default
    const metadata = await context.polkadotApi.query.assets.metadata(assetId.toU8a());
    expect(metadata.name.toString()).to.eq("0x");

    // assetDetails is None
    const assetDetails = await context.polkadotApi.query.assets.asset(assetId.toU8a());
    expect(assetDetails.isNone).to.eq(true);
  });
});
