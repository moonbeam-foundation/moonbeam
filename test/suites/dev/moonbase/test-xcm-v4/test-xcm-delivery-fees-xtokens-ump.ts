import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith, GLMR } from "@moonwall/util";
import { getLastSentUmpMessageFee } from "../../../../helpers";

describeSuite({
  id: "D014134",
  title: "XCM Delivery fees - Send upward message through Xtokens pallet",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const txByteFee = 100n;

    it({
      id: "T01",
      title: "Should succeed calling transfer",
      test: async function () {
        // 32 byte account
        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";

        await context.createBlock(
          context
            .polkadotJs()
            .tx.xTokens.transfer(
              "SelfReserve",
              100n * GLMR,
              {
                V4: {
                  parents: 1n,
                  interior: {
                    X1: [{ AccountId32: { network: null, key: destination_address } }],
                  },
                },
              } as any,
              {
                Limited: { refTime: 4000000000, proofSize: 64 * 1024 },
              }
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
        //		XCM Msg Bytes: 89
        expect(deliveryFee).to.be.equal(100000000008900n);
      },
    });
  },
});
