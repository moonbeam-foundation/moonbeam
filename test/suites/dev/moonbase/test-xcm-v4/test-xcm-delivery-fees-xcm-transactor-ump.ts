import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { fromBytes } from "viem";
import { getLastSentUmpMessageFee } from "../../../../helpers";

describeSuite({
  id: "D014132",
  title: "XCM Delivery fees - Send upward message through XCM Transactor",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const txByteFee = 100n;

    it({
      id: "T01",
      title: "Should succeed calling transactThroughSigned",
      test: async function () {
        // Use the relay as destination
        const dest = {
          V4: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
        };

        const transactCall = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeights = context
          .polkadotJs()
          .createType("PalletXcmTransactorTransactWeights", {
            transactRequiredWeightAtMost: { refTime: 10000, proofSize: 10000 },
            overallWeight: { Limited: { refTime: 10000, proofSize: 10000 } },
          });

        const fee = context.polkadotJs().createType("PalletXcmTransactorCurrencyPayment", {
          currency: {
            AsMultiLocation: {
              V4: {
                parents: 1,
                interior: {
                  Here: null,
                },
              },
            },
          },
          feeAmount: 10000,
        });

        await context.createBlock(
          context
            .polkadotJs()
            .tx.xcmTransactor.transactThroughSigned(
              dest,
              fee as any,
              transactCall,
              transactWeights as any,
              false
            )
            .signAsync(alith),
          { allowFailures: false }
        );

        const deliveryFee = await getLastSentUmpMessageFee(context, baseDelivery, txByteFee);

        // Delivery fee (total):
        //    DeliveryFeeFactor * [BaseDeliveryFee + (TransactionByteFee * XCM Msg Bytes)]
        //
        //    DeliveryFeeFactor: 1
        // 		BaseDeliveryFee: 100000000000000
        // 		TransactionByteFee: 100
        //		XCM Msg Bytes: 51
        expect(deliveryFee).to.be.equal(100000000005100n);
      },
    });
  },
});
