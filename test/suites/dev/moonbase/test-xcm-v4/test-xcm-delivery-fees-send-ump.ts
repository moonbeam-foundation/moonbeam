import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import {
  MultiLocation,
  extractPaidDeliveryFees,
  getLastSentUmpMessageFee,
  XcmFragment,
} from "../../../../helpers/xcm";

describeSuite({
  id: "D014131",
  title: "XCM Delivery fees - Send upward message",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const txByteFee = 100n;

    it({
      id: "T01",
      title: "Should succeed calling PolkadotXcm XCM send upwards",
      test: async function () {
        const xcmMessage = new XcmFragment({
          assets: [],
        })
          .clear_origin()
          .as_v4();

        const destMultilocation: MultiLocation = {
          parents: 1,
          interior: { Here: null },
        };

        const dest = {
          V4: destMultilocation,
        };
        const xcmSendTx = context.polkadotJs().tx.polkadotXcm.send(dest, xcmMessage);

        await context.createBlock(await xcmSendTx.signAsync(alith), { allowFailures: false });

        const fee = await getLastSentUmpMessageFee(context, baseDelivery, txByteFee);
        const paid = await extractPaidDeliveryFees(context);
        expect(paid).to.be.equal(fee);
      },
    });
  },
});
