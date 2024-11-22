import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith, ethan } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";


import {
  AssetMetadata,
  PARA_1000_SOURCE_LOCATION,
  PARA_1001_SOURCE_LOCATION,
  TestAsset,
  foreignAssetBalance,
  registerAndFundAsset,
  verifyLatestBlockFees,
  mockAssetBalance,
} from "../../../../helpers/index.js";

import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress20,
  injectHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";

export const InterlayAssetMetadata: AssetMetadata = {
  name: "INTR",
  symbol: "INTR",
  decimals: 12n,
  isFrozen: false,
};

export const MaticAssetMetadata: AssetMetadata = {
  name: "MATIC",
  symbol: "MATIC",
  decimals: 12n,
  isFrozen: false,
};

const DEFAULT_ADDRESS = "0x0101010101010101010101010101010101010101";

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

    const initialBalance: bigint = 10_000_000_000_000n;
    const xcMaticToSend = 3_500_000_000n;

    const xcIntrAsset: TestAsset = {
      id: 1000100010001000n,
      location: PARA_1000_SOURCE_LOCATION,
      metadata: InterlayAssetMetadata,
      relativePrice: 500_000,
    };

    const xcMaticAsset: TestAsset = {
      id: 1001100110011001n,
      location: PARA_1001_SOURCE_LOCATION,
      metadata: MaticAssetMetadata,
      relativePrice: 600_000,
    };

    beforeAll(async () => {
      api = context.polkadotJs();

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
        context,
        DEFAULT_ADDRESS,
        1000
      );

      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;

      // Register foreign asset used to pay fees (i.e. xcINTR)
      await registerAndFundAsset(context, xcIntrAsset, initialBalance, descendAddress);

      // Register foreign asset used to transfer(i.e.xcMatic)
      await registerAndFundAsset(context, xcMaticAsset, initialBalance, descendAddress);
      await mockAssetBalance(context, initialBalance, BigInt(xcIntrAsset.id), alith, ethan.address);
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

        const xcIntrBalanceBefore = await foreignAssetBalance(
          context,
          BigInt(xcIntrAsset.id),
          descendAddress
        );

        const xcMaticBalanceBefore = await foreignAssetBalance(
          context,
          BigInt(xcMaticAsset.id),
          descendAddress
        );

        // Simulate reception of an incoming XCM message and create block to execute it
        await injectHrmpMessageAndSeal(context, 1000, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const xcIntrBalanceAfter = await foreignAssetBalance(
          context,
          BigInt(xcIntrAsset.id),
          descendAddress
        );

        const xcMaticBalanceAfter = await foreignAssetBalance(
          context,
          BigInt(xcMaticAsset.id),
          descendAddress
        );

        const xcMaticReceivedEthan = await foreignAssetBalance(
          context,
          BigInt(xcMaticAsset.id),
          ethan.address as `0x${string}`
        );

        // Check that xcIntr where debited from Alith's descend address to pay the fees of the XCM execution
        expect(xcMaticBalanceBefore - xcMaticBalanceAfter).to.be.eq(xcMaticToSend);
        expect(initialBalance - xcMaticReceivedEthan).to.be.eq(xcMaticToSend);
        expect(xcIntrBalanceBefore - xcIntrBalanceAfter).to.be.eq(500_000_000n);
      },
    });
  },
});
