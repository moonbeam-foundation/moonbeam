import { beforeAll, describeSuite, expect } from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import {
  XcmFragment,
  registerForeignAsset,
  relayAssetMetadata,
  RELAY_SOURCE_LOCATION,
  addAssetToWeightTrader,
} from "../../../../helpers";

describeSuite({
  id: "D024120",
  title: "XCM - XcmPaymentApi",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;
    const assetId = 1n;

    beforeAll(async function () {
      polkadotJs = context.polkadotJs();

      await registerForeignAsset(
        context,
        assetId,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata as any
      );

      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 1_000_000_000_000_000_000n, context);
    });

    it({
      id: "T01",
      title: "Should succeed calling XcmPaymentApi methods",
      test: async function () {
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() === "Balances")!
          .index.toNumber();

        const allowedAssets = await polkadotJs.call.xcmPaymentApi.queryAcceptablePaymentAssets(3) as any;

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

        // Foreign asset registered in Weight Trader with a 1-1 relative price to the native asset
        expect(BigInt(weightToForeignFee.asOk.toJSON())).to.eq(125_000_000_000_000n);

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
          xcmMessage
        ) as any;
        expect(deliveryFees.isOk).to.be.true;
        // No delivery fees set for now
        expect(deliveryFees.asOk.toHuman()["V3"]).to.be.empty;
      },
    });
  },
});
