import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { type ApiPromise, WsProvider } from "@polkadot/api";
import { XcmFragment } from "../../../../helpers/xcm.js";
import {
  registerForeignAsset,
  addAssetToWeightTrader,
  relayAssetMetadata,
  RELAY_SOURCE_LOCATION,
} from "../../../../helpers/assets.js";

describeSuite({
  id: "D024212",
  title: "XCM - XcmPaymentApi",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;
    const relayAssetId = 1n;

    beforeAll(async function () {
      polkadotJs = context.polkadotJs();

      await registerForeignAsset(
        context,
        relayAssetId,
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
    });

    it({
      id: "T01",
      title: "Should succeed calling XcmPaymentApi methods",
      test: async function () {
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() === "Balances")!
          .index.toNumber();

        const allowedAssets = await polkadotJs.call.xcmPaymentApi.queryAcceptablePaymentAssets(3);

        expect(allowedAssets.isOk).to.be.true;
        // Should include the native asset + the foreign one
        expect(allowedAssets.asOk.toJSON().length).to.be.equal(2);

        const weightToNativeFee = await polkadotJs.call.xcmPaymentApi.queryWeightToAssetFee(
          {
            refTime: 10_000_000_000n,
            proofSize: 80_000n,
          },
          {
            V3: {
              Concrete: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
            },
          }
        );

        expect(weightToNativeFee.isOk).to.be.true;
        // 0.0005 GLMR
        expect(BigInt(weightToNativeFee.asOk.toJSON())).to.eq(125_000_000_000_000n);

        const weightToForeignFee = await polkadotJs.call.xcmPaymentApi.queryWeightToAssetFee(
          {
            refTime: 10_000_000_000n,
            proofSize: 0n,
          },
          {
            V3: {
              Concrete: { parents: 1, interior: "Here" },
            },
          }
        );

        expect(weightToForeignFee.isOk).to.be.true;

        // (unitsPerSec * weight.ref_time()) / WEIGHT_REF_TIME_PER_SECOND
        // (20_000_000_000 * 10_000_000_000) / 1_000_000_000_000
        expect(BigInt(weightToForeignFee.asOk.toJSON())).to.eq(200_000_000n);

        const transactWeightAtMost = {
          refTime: 500_000_000n,
          proofSize: 20000n,
        };

        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: 1000000000n,
            },
          ],
          weight_limit: {
            refTime: 40000000000n,
            proofSize: 110000n,
          },
        })
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: transactWeightAtMost,
              call: {
                encoded: polkadotJs.tx.balances
                  .transferAllowDeath("0x0000000000000000000000000000000000000000", 1000000000n)
                  .method.toHex(),
              },
            },
          })
          .as_v3();

        const weightMessage = await polkadotJs.call.xcmPaymentApi.queryXcmWeight(xcmMessage);
        expect(weightMessage.isOk).to.be.true;
        expect(weightMessage.asOk.refTime.toBigInt() > transactWeightAtMost.refTime).to.be.true;
        expect(weightMessage.asOk.proofSize.toBigInt() > transactWeightAtMost.proofSize).to.be.true;

        const dest = {
          V3: {
            parents: 1,
            interior: "Here",
          },
        };

        const deliveryFees = await polkadotJs.call.xcmPaymentApi.queryDeliveryFees(
          dest,
          xcmMessage,
          {
            V3: {
              Concrete: { parents: 1, interior: "Here" },
            },
          },
        );

        // No delivery fees set for now
        expect(deliveryFees.isOk).to.be.false;
      },
    });
  },
});
