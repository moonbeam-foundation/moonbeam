import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import { ConstantStore } from "../../../../helpers";

describeSuite({
  id: "D021501",
  title: "Genesis Fee Multiplier",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should start with genesis value",
      test: async () => {
        const { specVersion } = await context.polkadotJs().consts.system.version;
        const GENESIS_BASE_FEE = ConstantStore(context).GENESIS_BASE_FEE.get(
          specVersion.toNumber()
        );
        const initialValue = (
          await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
        ).toBigInt();
        expect(initialValue).to.equal(8_000_000_000_000_000_000n);

        const gasPrice = await context.viem().getGasPrice();
        expect(gasPrice).to.eq(GENESIS_BASE_FEE);
      },
    });
  },
});
