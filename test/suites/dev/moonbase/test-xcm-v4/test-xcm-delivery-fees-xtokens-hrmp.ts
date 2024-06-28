import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith, GLMR } from "@moonwall/util";
import { getLastSentHrmpMessageFee, mockHrmpChannelExistanceTx } from "../../../../helpers";

describeSuite({
  id: "D014135",
  title: "XCM Delivery fees - Send horizontal message through Xtokens pallet",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const txByteFee = 100n;
    const destParaId = 2000;

    it({
      id: "T01",
      title: "Should succeed calling transfer",
      test: async function () {
        const mockHrmp2000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destParaId, 1000, 102400, 102400));
        let aliceNonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();

        // 32 byte account
        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";

        const xTokensTx = context.polkadotJs().tx.xTokens.transfer(
          "SelfReserve",
          100n * GLMR,
          {
            V4: {
              parents: 1n,
              interior: {
                X2: [
                  { Parachain: destParaId },
                  { AccountId32: { network: null, key: destination_address } },
                ],
              },
            },
          } as any,
          {
            Limited: { refTime: 4000000000, proofSize: 64 * 1024 },
          }
        );

        await context.createBlock(
          [
            await mockHrmp2000Tx.signAsync(alith, { nonce: aliceNonce++ }),
            await xTokensTx.signAsync(alith, { nonce: aliceNonce++ }),
          ],
          { allowFailures: false }
        );

        const deliveryFee = await getLastSentHrmpMessageFee(
          context,
          destParaId,
          baseDelivery,
          txByteFee
        );

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
