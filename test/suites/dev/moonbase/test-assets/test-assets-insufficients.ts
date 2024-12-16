import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith, baltathar, generateKeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import type { u128 } from "@polkadot/types";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { BN } from "@polkadot/util";
import { mockOldAssetBalance } from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;
const ARBITRARY_TRANSFER_AMOUNT = 10000000000000n;

describeSuite({
  id: "D010104",
  title: "Pallet Assets - Sufficient tests: is_sufficient to false",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let assetId: u128;
    let api: ApiPromise;
    const freshAccount = generateKeyringPair();

    beforeAll(async () => {
      api = context.polkadotJs();
      assetId = api.createType("u128", ARBITRARY_ASSET_ID);
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance: PalletAssetsAssetAccount = api.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      const assetDetails: PalletAssetsAssetDetails = api.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: false,
        minBalance: 1,
      });

      await mockOldAssetBalance(context, assetBalance, assetDetails, alith, assetId, ALITH_ADDRESS);

      await context.createBlock();
      const alithBalance = await api.query.assets.account(assetId.toU8a(), ALITH_ADDRESS);
      expect(alithBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    });

    it({
      id: "T01",
      title: "Send MOVR and assets to an account, then drain assets, dont drain MOVR",
      test: async function () {
        // We are going to use a fresh account here, since we mocked the asset balance

        // We cannot transfer to freshAccount, since sufficient is false
        await context.createBlock(
          api.tx.assets.transfer(assetId, freshAccount.address, ARBITRARY_TRANSFER_AMOUNT),
          { allowFailures: true }
        );

        expect(
          (await api.query.system.account(freshAccount.address as string)).sufficients.toBigInt()
        ).to.eq(0n);
        // Providers should still be 0
        expect(
          (await api.query.system.account(freshAccount.address as string)).providers.toBigInt()
        ).to.eq(0n);

        // Lets transfer it the native token. We want to transfer enough to cover for a future fee.
        const fee = (
          await api.tx.assets
            .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
            .paymentInfo(freshAccount)
        ).partialFee.toBigInt();

        // We transfer Balances, which should increase provider
        await context.createBlock(api.tx.balances.transferAllowDeath(freshAccount.address, fee));

        expect(
          (await api.query.system.account(freshAccount.address as string)).sufficients.toBigInt()
        ).to.eq(0n);
        // Providers should now be 1
        expect(
          (await api.query.system.account(freshAccount.address as string)).providers.toBigInt()
        ).to.eq(1n);

        // We now can transfer assets to freshAccount, since it has a provider
        await api.tx.assets
          .transfer(assetId, freshAccount.address, ARBITRARY_TRANSFER_AMOUNT)
          .signAndSend(alith);

        await context.createBlock();

        expect(
          (await api.query.system.account(freshAccount.address as string)).sufficients.toBigInt()
        ).to.eq(0n);

        expect(
          (await api.query.system.account(freshAccount.address as string)).providers.toBigInt()
        ).to.eq(1n);

        expect(
          (await api.query.system.account(freshAccount.address as string)).consumers.toBigInt()
        ).to.eq(1n);

        // When we execute transaction, both MOVR and Assets should be drained
        await context.createBlock(
          api.tx.assets
            .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
            .signAsync(freshAccount)
        );

        const freshAccountBalance = await api.query.assets.account(
          assetId.toU8a(),
          freshAccount.address as string
        );
        expect(freshAccountBalance.isNone).to.equal(true);

        const freshSystemAccount = await api.query.system.account(freshAccount.address as string);
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
      },
    });
  },
});
