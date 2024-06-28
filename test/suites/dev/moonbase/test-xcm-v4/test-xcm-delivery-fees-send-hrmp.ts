import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import {
  MultiLocation,
  extractPaidDeliveryFees,
  getLastSentHrmpMessageFee,
  XcmFragment,
  mockHrmpChannelExistanceTx,
} from "../../../../helpers/xcm";

describeSuite({
  id: "D014130",
  title: "XCM Delivery fees - Send HRMP message",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const destinationPara = 2000;
    const txByteFee = 100n;

    it({
      id: "T01",
      title: "Should succeed calling XCM PolkadotXcm send horizontal",
      test: async function () {
        const mockHrmp2000Tx = context
          .polkadotJs()
          .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, destinationPara, 1000, 102400, 102400));
        let aliceNonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();

        const xcmMessage = new XcmFragment({
          assets: [],
        })
          .clear_origin()
          .as_v4();

        const destMultilocation = {
          parents: 1,
          interior: {
            X1: [
              {
                Parachain: destinationPara,
              },
            ],
          },
        };

        const dest = {
          V4: destMultilocation,
        };
        const tx = context.polkadotJs().tx.polkadotXcm.send(dest, xcmMessage);

        await context.createBlock(
          [
            await mockHrmp2000Tx.signAsync(alith, { nonce: aliceNonce++ }),
            await tx.signAsync(alith, { nonce: aliceNonce++ }),
          ],
          { allowFailures: false }
        );

        const fee = await getLastSentHrmpMessageFee(
          context,
          destinationPara,
          baseDelivery,
          txByteFee
        );
        const paid = await extractPaidDeliveryFees(context);
        expect(paid).to.be.equal(fee);
      },
    });
  },
});
