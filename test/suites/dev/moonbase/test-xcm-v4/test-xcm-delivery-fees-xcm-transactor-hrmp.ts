import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { fromBytes } from "viem";
import { getLastSentHrmpMessageFee, mockHrmpChannelExistanceTx } from "../../../../helpers";

describeSuite({
  id: "D014133",
  title: "XCM Delivery fees - Send horizontal message through XCM Transactor",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const txByteFee = 100n;
    const destParaId = 2000;

    it({
      id: "T01",
      title: "Should succeed calling transactThroughSigned",
      test: async function () {
        const mockHrmp2000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destParaId, 1000, 102400, 102400));
        let aliceNonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();

        // Parachain 2000 as destination
        const dest = {
          V4: {
            parents: 1,
            interior: {
              X1: [{ Parachain: destParaId }],
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
                  X2: [{ Parachain: destParaId }, { PalletInstance: 3 }],
                },
              },
            },
          },
          feeAmount: 10000,
        });

        let transactorTx = context
          .polkadotJs()
          .tx.xcmTransactor.transactThroughSigned(
            dest,
            fee as any,
            transactCall,
            transactWeights as any,
            false
          );

        await context.createBlock(
          [
            await mockHrmp2000Tx.signAsync(alith, { nonce: aliceNonce++ }),
            await transactorTx.signAsync(alith, { nonce: aliceNonce++ }),
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
        //		XCM Msg Bytes: 55
        expect(deliveryFee).to.be.equal(100000000005500n);
      },
    });
  },
});
