import "@moonbeam-network/api-augment";

import { u128 } from "@polkadot/types";
import { BN } from "@polkadot/util";
import { expect } from "chai";

import { alith, baltathar, generateKeyingPair } from "../../util/accounts";
import { mockAssetBalance } from "../../util/assets";
import { GLMR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;
const ARBITRARY_TRANSFER_AMOUNT = 10000000000000n;

describeDevMoonbeam(
  "Pallet Assets - Sufficient tests: is_sufficient to true",
  (context) => {
    let assetId: u128;
    const freshAccount = generateKeyingPair();

    before("Setup contract and mock balance", async () => {
      assetId = context.polkadotApi.createType("u128", ARBITRARY_ASSET_ID);
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: true,
        minBalance: 1,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        true
      );

      await context.createBlock();
      const alithBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        alith.address
      );
      expect(alithBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    });

    it("Send MOVR and assets to an account, then drain assets, then MOVR", async function () {
      // We are going to use a fresh account here, since we mocked the asset balance

      // We transfer Assets to freshAccount, which should increase sufficients
      await context.createBlock(
        context.polkadotApi.tx.assets.transfer(
          assetId,
          freshAccount.address,
          ARBITRARY_TRANSFER_AMOUNT
        )
      );

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should still be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(0n);

      // We transfer a good amount to be able to pay for fees
      await context.createBlock(
        context.polkadotApi.tx.balances.transfer(freshAccount.address, 1n * GLMR)
      );

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);

      // Providers should now be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // Let's drain assets
      await context.createBlock(
        context.polkadotApi.tx.assets
          .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
          .signAsync(freshAccount)
      );

      // Lets drain native token
      // First calculate fee
      // Then grab balance of freshAccount
      // Then we just transfer out balance of freshAccount - fee
      const fee = (
        await context.polkadotApi.tx.balances
          .transfer(alith.address, 1n * GLMR)
          .paymentInfo(freshAccount)
      ).partialFee.toBigInt();

      const freshAccountBalanceNativeToken = (
        await context.polkadotApi.query.system.account(freshAccount.address)
      ).data.free.toBigInt();

      await context.createBlock(
        context.polkadotApi.tx.balances
          .transfer(baltathar.address, freshAccountBalanceNativeToken - fee)
          .signAsync(freshAccount)
      );

      const freshAccountBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        freshAccount.address
      );
      expect(freshAccountBalance.isNone).to.equal(true);

      // Sufficients should go to 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // Nonce should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // But balance of MOVR should be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).data.free.toBigInt()
      ).to.eq(0n);
    });
  },
  "Legacy",
  "moonbase",
  true
);

describeDevMoonbeam(
  "Pallet Assets - Sufficient tests: is_sufficient to true",
  (context) => {
    let assetId: u128;
    const freshAccount = generateKeyingPair();

    before("Setup contract and mock balance", async () => {
      assetId = context.polkadotApi.createType("u128", ARBITRARY_ASSET_ID);
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });
      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: true,
        minBalance: 1,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        true
      );

      await context.createBlock();
      const alithBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        alith.address
      );
      expect(alithBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    });

    it("Send MOVR and assets to an account, then drain assets, dont drain MOVR", async function () {
      // We are going to use a fresh account here, since we mocked the asset balance

      // We transfer Assets to freshAccount, which should increase sufficients

      await context.createBlock(
        context.polkadotApi.tx.assets.transfer(
          assetId,
          freshAccount.address,
          ARBITRARY_TRANSFER_AMOUNT
        )
      );

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should still be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(0n);

      // Lets transfer it the native token. We want to transfer enough to cover for a future fee.
      const fee = (
        await context.polkadotApi.tx.assets
          .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
          .paymentInfo(freshAccount)
      ).partialFee.toBigInt();

      await context.createBlock(
        context.polkadotApi.tx.balances.transfer(freshAccount.address, fee)
      );

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should now be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // What happens now when we execute such transaction? both MOVR and Assets should be drained.
      await context.createBlock(
        context.polkadotApi.tx.assets
          .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
          .signAsync(freshAccount)
      );

      const freshAccountBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        freshAccount.address
      );
      expect(freshAccountBalance.isNone).to.equal(true);

      // Sufficients should go to 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // Nonce should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // But balance of MOVR should be 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).data.free.toBigInt() > 0n
      ).to.eq(true);
    });
  },
  "Legacy",
  "moonbase",
  true
);

describeDevMoonbeam(
  "Pallet Assets - Sufficient tests: is_sufficient to false",
  (context) => {
    let assetId: u128;
    const freshAccount = generateKeyingPair();

    before("Setup contract and mock balance", async () => {
      assetId = context.polkadotApi.createType("u128", ARBITRARY_ASSET_ID);
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: false,
        minBalance: 1,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        false
      );

      await context.createBlock();
      const alithBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        alith.address
      );
      expect(alithBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    });

    it("Send MOVR and assets to an account, then drain assets, dont drain MOVR", async function () {
      // We are going to use a fresh account here, since we mocked the asset balance

      // We cannot transfer to freshAccount, since sufficient is false
      await context.createBlock(
        context.polkadotApi.tx.assets.transfer(
          assetId,
          freshAccount.address,
          ARBITRARY_TRANSFER_AMOUNT
        )
      );

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should still be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(0n);

      // Lets transfer it the native token. We want to transfer enough to cover for a future fee.
      const fee = (
        await context.polkadotApi.tx.assets
          .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
          .paymentInfo(freshAccount)
      ).partialFee.toBigInt();

      // We transfer Balances, which should increase provider
      await context.createBlock(
        context.polkadotApi.tx.balances.transfer(freshAccount.address, fee)
      );

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should now be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // We now can transfer assets to freshAccount, since it has a provider
      await context.polkadotApi.tx.assets
        .transfer(assetId, freshAccount.address, ARBITRARY_TRANSFER_AMOUNT)
        .signAndSend(alith);

      await context.createBlock();

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);

      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).consumers.toBigInt()
      ).to.eq(1n);

      // What happens now when we execute such transaction? both MOVR and Assets should be drained.
      await context.createBlock(
        context.polkadotApi.tx.assets
          .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
          .signAsync(freshAccount)
      );

      const freshAccountBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        freshAccount.address
      );
      expect(freshAccountBalance.isNone).to.equal(true);

      const freshSystemAccount = await context.polkadotApi.query.system.account(
        freshAccount.address
      );
      // Sufficients should be 0
      expect(freshSystemAccount.sufficients.toBigInt()).to.eq(0n);

      // Consumers should be 0
      expect(freshSystemAccount.consumers.toBigInt()).to.eq(0n);

      // Providers should be 1
      expect(freshSystemAccount.providers.toBigInt()).to.eq(1n);

      // Nonce should be 1
      expect(freshSystemAccount.providers.toBigInt()).to.eq(1n);

      // But balance of MOVR should be 0
      expect(freshSystemAccount.data.free.toBigInt() > 0n).to.be.true;
    });
  },
  "Legacy",
  "moonbase",
  true
);
