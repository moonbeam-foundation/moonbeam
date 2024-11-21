import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith, ethan } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { u128 } from "@polkadot/types";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { hexToBigInt } from "@polkadot/util";
import {
  AssetMetadata,
  PARA_1000_SOURCE_LOCATION,
  PARA_1001_SOURCE_LOCATION,
  mockOldAssetBalance,
  registerOldForeignAsset,
  verifyLatestBlockFees,
} from "../../../../helpers/index.js";
import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress20,
  injectHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";

export const InterlayAsset: AssetMetadata = {
  name: "INTR",
  symbol: "INTR",
  decimals: 12n,
  isFrozen: false,
};

export const MaticAsset: AssetMetadata = {
  name: "MATIC",
  symbol: "MATIC",
  decimals: 12n,
  isFrozen: false,
};

/**
 * We are going to test the following scenario:
 *
 * An account in the origin parachain, let's say Interlay, wants to send MATIC tokens from
 * Interlay to Moonbeam through XCM, and wants to pay the transaction with in the origin
 * chain's token, INTR, so they don't need to buy GLMR.
 *
 * From Moonbeam's point of view, this is an incoming transfer, and we need to test that
 * the XCM transaction went through, and the assets in the origin account were
 * succesfully deducted.
 *
 * For this we're going to create two foreign assets, xcINTR for paying fees and xcMATIC to be
 * transferred. And we are going to fund an account in the origin's chain asset.
 *
 * Parachain 1000 would be the location of the sending parachain (INTR).
 * Parachain 1001 would be the location of MATIC.
 *
 */
describeSuite({
  id: "D014137",
  title: "Mock XCM - Transfer some ERC20 token and pay with origin chain's token",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let api: ApiPromise;
    let xcIntrAssetId: u128;
    let xcMaticAssetId: u128;

    const xcIntrUnitsPerSecond: number = 10;
    const xcMaticUnitsPerSecond: number = 12;
    const initialSenderBalance: bigint = 10_000_000_000_000n;
    const xcMaticToSend = 3_500_000_000n;

    beforeAll(async () => {
      api = context.polkadotJs();

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
        context,
        "0x0101010101010101010101010101010101010101",
        1000
      );
      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;

      // Register foreign asset used to pay fees (i.e. xcINTR)
      xcIntrAssetId = await registerAndFundAsset(
        context,
        xcIntrUnitsPerSecond,
        PARA_1000_SOURCE_LOCATION,
        initialSenderBalance,
        descendAddress
      );

      // Register foreign asset used to transfer (i.e. xcMatic)
      xcMaticAssetId = await registerAndFundAsset(
        context,
        xcMaticUnitsPerSecond,
        PARA_1001_SOURCE_LOCATION,
        initialSenderBalance,
        descendAddress
      );
    });

    it({
      id: "T01",
      title: "should receive foreign asset transfer, paying fees in origin chain's foreign asset",
      test: async function () {
        // 3. Build incoming XCM message
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: PARA_1000_SOURCE_LOCATION.Xcm,
              fungible: 1_000_000_000n,
            },
            {
              multilocation: PARA_1001_SOURCE_LOCATION.Xcm,
              fungible: xcMaticToSend,
            },
          ],
          weight_limit: {
            refTime: 50_000_000_000n,
            proofSize: 150000n,
          },
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .deposit_asset_definite(
            PARA_1001_SOURCE_LOCATION.Xcm,
            xcMaticToSend,
            ethan.address as `0x${string}`
          )
          .deposit_asset_definite(
            PARA_1000_SOURCE_LOCATION.Xcm,
            500_000_000n,
            descendAddress as `0x${string}`
          )
          .as_v4();

        await verifyLatestBlockFees(context);

        const xcIntrBalanceBefore = (await api.query.assets.account(xcIntrAssetId, descendAddress))
          .unwrap()
          .balance.toBigInt();

        const xcMaticBalanceBefore = (
          await api.query.assets.account(xcMaticAssetId, descendAddress)
        )
          .unwrap()
          .balance.toBigInt();

        // Simulate reception of an incoming XCM message and create block to execute it
        await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const xcIntrBalanceAfter = (await api.query.assets.account(xcIntrAssetId, descendAddress))
          .unwrap()
          .balance.toBigInt();

        const xcMaticBalanceAfter = (await api.query.assets.account(xcMaticAssetId, descendAddress))
          .unwrap()
          .balance.toBigInt();

        const xcMaticReceivedEthan = (await api.query.assets.account(xcMaticAssetId, ethan.address))
          .unwrap()
          .balance.toBigInt();

        // Check that xcIntr where debited from Alith's descend address to pay the fees of the XCM execution
        expect(xcMaticBalanceBefore - xcMaticBalanceAfter).to.be.eq(xcMaticToSend);
        expect(xcMaticReceivedEthan).to.be.eq(xcMaticToSend);
        expect(xcIntrBalanceBefore - xcIntrBalanceAfter).to.be.eq(500_000_000n);
      },
    });
  },
});

async function registerAndFundAsset(
  context: any,
  unitsPerSecond: number,
  assetLocation: any,
  balance: bigint,
  address: string
) {
  const api = context.polkadotJs();

  const { registeredAssetId } = await registerOldForeignAsset(
    context,
    assetLocation,
    InterlayAsset as any,
    unitsPerSecond
  );

  const initialBalance = api.createType("Balance", balance);
  const assetId = api.createType("u128", hexToBigInt(registeredAssetId as `0x${string}`));

  const assetBalance: PalletAssetsAssetAccount = api.createType("PalletAssetsAssetAccount", {
    balance: initialBalance,
  });
  const assetDetails: PalletAssetsAssetDetails = api.createType("PalletAssetsAssetDetails", {
    supply: initialBalance,
  });

  await mockOldAssetBalance(context, assetBalance, assetDetails, alith, assetId, address);

  return assetId;
}
