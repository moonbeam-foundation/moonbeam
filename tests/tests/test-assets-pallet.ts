import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN } from "@polkadot/util";
import { ALITH, ALITH_PRIV_KEY, BALTATHAR } from "../util/constants";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { mockAssetBalance } from "./test-precompile/test-precompile-assets-erc20";

const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

describeDevMoonbeam("Pallet Assets Pallet - assets transfer", (context) => {
  let sudoAccount, assetId;
  before("Test querying asset", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });

    assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
      supply: balance,
    });

    await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId, ALITH);

    await createBlockWithExtrinsic(
      context,
      sudoAccount,
      context.polkadotApi.tx.assets.transfer(assetId, BALTATHAR, 1000)
    );
  });

  it("should query asset balance", async function () {
    // Baltathar balance is 1000
    let baltatharBalance = (await context.polkadotApi.query.assets.account(
      assetId,
      BALTATHAR
    )) as any;
    expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
  });
});

describeDevMoonbeam("Pallet Assets Pallet - asset destruction", (context) => {
  let sudoAccount, assetId;
  before("Test querying asset", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });

    assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
      supply: balance,
    });

    await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId, ALITH);

    await createBlockWithExtrinsic(
      context,
      sudoAccount,
      context.polkadotApi.tx.assets.transfer(assetId, BALTATHAR, 1000)
    );
  });

  it("should make sure everything related to the asset is destroyed", async function () {
    // We first create the witness
    const assetDestroyWitness = context.polkadotApi.createType("PalletAssetsDestroyWitness", {
      accounts: 1,
      sufficients: 1,
      approvals: 0,
    });

    let metadataBefore = (await context.polkadotApi.query.assets.metadata(assetId)) as any;

    // Name is equal to "DOT" in hex
    expect(metadataBefore.name.toString()).to.eq("0x444f54");

    // Baltathar has its balance
    let baltatharBalanceBefore = (await context.polkadotApi.query.assets.account(
      assetId,
      BALTATHAR
    )) as any;
    expect(baltatharBalanceBefore.unwrap()["balance"].eq(new BN(1000))).to.equal(true);

    // assetDetails before in non-empty
    let assetDetailsBefore = (await context.polkadotApi.query.assets.asset(assetId)) as any;
    expect(assetDetailsBefore.isNone).to.eq(false);

    // Destroy asset
    await createBlockWithExtrinsic(
      context,
      sudoAccount,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assets.destroy(assetId, assetDestroyWitness)
      )
    );

    // Baltathar balance is None
    let baltatharBalance = (await context.polkadotApi.query.assets.account(
      assetId,
      BALTATHAR
    )) as any;
    expect(baltatharBalance.isNone).to.eq(true);

    // metadata is default
    let metadata = (await context.polkadotApi.query.assets.metadata(assetId)) as any;
    expect(metadata.name.toString()).to.eq("0x");

    // assetDetails is None
    let assetDetails = (await context.polkadotApi.query.assets.asset(assetId)) as any;
    expect(assetDetails.isNone).to.eq(true);
  });
});
