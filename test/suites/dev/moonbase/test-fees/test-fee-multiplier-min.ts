import "@moonbeam-network/api-augment/moonbase";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { nToHex } from "@polkadot/util";

// Note on the values from 'transactionPayment.nextFeeMultiplier': this storage item is actually a
// FixedU128, which is basically a u128 with an implicit denominator of 10^18. However, this
// denominator is omitted when it is queried through the API, leaving some very large numbers.
//
// To make sense of them, basically remove 18 zeroes (divide by 10^18). This will give you the
// number used internally by transaction-payment for fee calculations.
describeSuite({
  id: "D021503",
  title: "Min Fee Multiplier",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeEach(async () => {
      const MULTIPLIER_STORAGE_KEY = context
        .polkadotJs()
        .query.transactionPayment.nextFeeMultiplier.key(0)
        .toString();

      // set transaction-payment's multiplier to something above max in storage. on the next round,
      // it should enforce its upper bound and reset it.
      await context
        .polkadotJs()
        .tx.sudo.sudo(
          context
            .polkadotJs()
            .tx.system.setStorage([
              [MULTIPLIER_STORAGE_KEY, nToHex(1n, { isLe: true, bitLength: 128 })],
            ])
        )
        .signAndSend(alith);
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should enforce lower bound",
      test: async function () {
        const MULTIPLIER_STORAGE_KEY = context
          .polkadotJs()
          .query.transactionPayment.nextFeeMultiplier.key(0)
          .toString();

        // we set it to u128_max, but the max should have been enforced in on_finalize()
        const multiplier = (
          await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
        ).toBigInt();
        expect(multiplier).to.equal(100_000_000_000_000_000n);

        const gasPrice = await context.viem().getGasPrice();
        expect(gasPrice).to.eq(31_250_000n);
      },
    });
  },
});
