import "@moonbeam-network/api-augment";
import { alith, beforeAll, describeSuite, expect, generateKeyringPair } from "moonwall";
import { type ApiPromise, WsProvider } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { u128 } from "@polkadot/types";
import { hexToBigInt } from "@polkadot/util";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";
import {
  relayAssetMetadata,
  RELAY_SOURCE_LOCATION,
  registerForeignAsset,
  mockAssetBalance,
  addAssetToWeightTrader,
  foreignAssetBalance,
} from "../../../../helpers/assets.js";

describeSuite({
  id: "D024211",
  title: "XCM - XcmPaymentApi - Transact",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let polkadotJs: ApiPromise;
    let amountForFees: bigint;
    let amountForTransfer: bigint;
    let assetId: u128;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;
    const foreignAssetId = 1n;
    const weightLimit = {
      refTime: 40_000_000_000n,
      proofSize: 120_583n,
    };
    let weightToForeignFee: any;

    beforeAll(async () => {
      polkadotJs = context.polkadotJs();

      await registerForeignAsset(
        context,
        foreignAssetId,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata as any
      );

      // Calculate relative price: equivalent to 20000000000 unitsPerSecond
      const WEIGHT_REF_TIME_PER_SECOND = 1_000_000_000_000n;
      const nativeAmountPerSecond = await context
        .polkadotJs()
        .call.transactionPaymentApi.queryWeightToFee({
          refTime: WEIGHT_REF_TIME_PER_SECOND,
          proofSize: 0n,
        });

      const relativePriceDecimals = 18n;
      const relativePrice =
        (BigInt(nativeAmountPerSecond.toString()) * 10n ** relativePriceDecimals) / 20000000000n;

      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, relativePrice, context);

      // Fetch the exact amount of foreign fees that we will use given
      // the indicated weightLimit
      weightToForeignFee = await polkadotJs.call.xcmPaymentApi.queryWeightToAssetFee(weightLimit, {
        V3: {
          Concrete: { parents: 1, interior: "Here" },
        },
      });

      expect(weightToForeignFee.isOk).to.be.true;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      descendAddress = descendOriginAddress;
      sendingAddress = originAddress;

      random = generateKeyringPair();
      // Amount to use inside BuyExecution
      amountForFees = BigInt(weightToForeignFee.asOk.toJSON());
      // Amount to transfer to random address
      amountForTransfer = 1_000_000_000_000_000_000n;

      // Fund descendAddress with enough foreign asset to pay XCM execution fees
      await mockAssetBalance(context, amountForFees, foreignAssetId, alith, descendAddress);

      // We need to fund the descendAddress with both amounts.
      // This account takes care of paying the foreign fees and also transfering the
      // native tokens to the random address.
      await context.createBlock(
        polkadotJs.tx.balances.transferAllowDeath(descendAddress, amountForTransfer),
        { allowFailures: false }
      );

      const descendForeignBalance = await foreignAssetBalance(
        context,
        foreignAssetId,
        descendAddress
      );

      const descendNativeBalance = (
        await polkadotJs.query.system.account(descendAddress)
      ).data.free.toBigInt();
      expect(descendForeignBalance).to.eq(amountForFees);
      expect(descendNativeBalance).to.eq(amountForTransfer);
    });

    it({
      id: "T01",
      title: "Should de able to transact using the estimated foreign fees",
      test: async function () {
        // Build Transact encoded call
        const transferCall = polkadotJs.tx.balances.transferAllowDeath(
          random.address,
          amountForTransfer
        );
        const transferCallEncoded = transferCall?.method.toHex();

        // Build the XCM message with the corresponding weightLimit
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 1,
                interior: { Here: null },
              },
              fungible: amountForFees,
            },
          ],
          weight_limit: weightLimit,
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 1_000_000_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v5();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the random address received the transfer
        const testAccountBalance = (
          await polkadotJs.query.system.account(random.address)
        ).data.free.toBigInt();

        // Make sure the descendOrigin address has zero foreign balance now
        const testDescendBalance = await foreignAssetBalance(
          context,
          foreignAssetId,
          descendAddress
        );

        expect(testAccountBalance).to.eq(amountForTransfer);
        expect(testDescendBalance).to.eq(0n);
      },
    });
  },
});
