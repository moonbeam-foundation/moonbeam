import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith, baltathar, generateKeyringPair } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { u128 } from "@polkadot/types";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { BN } from "@polkadot/util";
import {
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  RELAY_SOURCE_LOCATION_V4,
  foreignAssetBalance,
  mockAssetBalance,
} from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;
const ARBITRARY_TRANSFER_AMOUNT = 10000000000000n;

describeSuite({
  id: "D010102",
  title: "Pallet Assets - Sufficient tests: is_sufficient to true",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: bigint;
    const freshAccount = generateKeyringPair();
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();
      assetId = ARBITRARY_ASSET_ID;
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = BigInt("100000000000000");
      const assetLocation = RELAY_SOURCE_LOCATION_V4;
      await mockAssetBalance(context, balance, assetId, assetLocation, alith, ALITH_ADDRESS);

      await context.createBlock();
      const alithBalance = foreignAssetBalance;
      expect(alithBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    });

    it({
      id: "T01",
      title: "Send MOVR and assets to an account, then drain assets, dont drain MOVR",
      test: async function () {
        // We are going to use a fresh account here, since we mocked the asset balance
        // We transfer Assets to freshAccount, which should increase sufficients

        await context.createBlock(
          api.tx.assets.transfer(assetId, freshAccount.address, ARBITRARY_TRANSFER_AMOUNT)
        );

        expect(
          (await api.query.system.account(freshAccount.address as string)).sufficients.toBigInt()
        ).to.eq(1n);
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

        await context.createBlock(api.tx.balances.transferAllowDeath(freshAccount.address, fee));

        expect(
          (await api.query.system.account(freshAccount.address as string)).sufficients.toBigInt()
        ).to.eq(1n);
        // Providers should now be 1
        expect(
          (await api.query.system.account(freshAccount.address as string)).providers.toBigInt()
        ).to.eq(1n);

        // When we execute transaction, both MOVR and Assets should be drained
        await context.createBlock(
          api.tx.assets
            .transfer(assetId, baltathar.address, ARBITRARY_TRANSFER_AMOUNT)
            .signAsync(freshAccount)
        );

        // Sufficients should go to 0
        expect(
          (await api.query.system.account(freshAccount.address as string)).sufficients.toBigInt()
        ).to.eq(0n);
        // Providers should be 1
        expect(
          (await api.query.system.account(freshAccount.address as string)).providers.toBigInt()
        ).to.eq(1n);

        // Nonce should be 1
        expect(
          (await api.query.system.account(freshAccount.address as string)).providers.toBigInt()
        ).to.eq(1n);

        // But balance of MOVR should be 0
        expect(
          (await api.query.system.account(freshAccount.address as string)).data.free.toBigInt() > 0n
        ).to.eq(true);
      },
    });
  },
});
