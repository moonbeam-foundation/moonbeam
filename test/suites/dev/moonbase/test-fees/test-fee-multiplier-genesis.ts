import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D011601",
  title: "Genesis Fee Multiplier",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should start with genesis value",
      test: async () => {
        const initialValue = (
          await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
        ).toBigInt();
        expect(initialValue).to.equal(8_000_000_000_000_000_000n);

        const gasPrice = await context.viem().getGasPrice();
        expect(gasPrice).to.eq(2_500_000_000n); // min gas price
      },
    });
  },
});
