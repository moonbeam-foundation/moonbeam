import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { createViemTransaction } from "@moonwall/util";
import { ConstantStore, GAS_LIMIT_POV_RATIO } from "../../../../helpers/constants";

describeSuite({
  id: "D011607",
  title: "Substrate Length Fees - Ethereum txn Interaction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not charge length fee for precompile from Ethereum txn",
      test: async () => {
        // we use modexp here because it allows us to send large-ish transactions
        const MODEXP_PRECOMPILE_ADDRESS = "0x0000000000000000000000000000000000000005";

        // directly call the modexp precompile with a large txn. this precompile lets us do two
        // things which are useful:
        //
        // 1. specify an input length up to 1024 for each of mod, exp, and base
        // 2. returns early and uses little gas (200) if all ore 0
        //
        // This allows us to create an Ethereum transaction whose fee is largely made up of
        // Ethereum's per-byte length fee (reminder: this is 4 gas for a 0 and 16 for any non-zero
        // byte). What we want to show is that this length fee is applied but our exponential
        // LengthToFee (part of our Substrate-based fees) is not applied.

        const tx = await createViemTransaction(context, {
          to: MODEXP_PRECOMPILE_ADDRESS,
          gas: BigInt(ConstantStore(context).EXTRINSIC_GAS_LIMIT),
          data: ("0x0000000000000000000000000000000000000000000000000000000000000004" + // base
            "0000000000000000000000000000000000000000000000000000000000000004" + // exp
            "0000000000000000000000000000000000000000000000000000000000000004" + // mod
            "0".repeat(2048) + // 2048 hex nibbles -> 1024 bytes
            "0".repeat(2048) +
            "0".repeat(2048)) as `0x${string}`,
        });

        const { result } = await context.createBlock(tx);
        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).toBe("success");

        // rough math on what the exponential LengthToFee modifier would do to this:
        // * input data alone is (3 * 1024) + (3 * 32) = 3168
        // * 3168 ** 3 = 31_794_757_632
        // * 31_794_757_632 / WEIGHT_PER_GAS = 1_271_790
        //
        // conclusion: the LengthToFee modifier is NOT involved
        expect(receipt.gasUsed, "gasUsed does not match manual calculation").toBeLessThan(
          1_271_790n
        );

        // furthermore, we can account for the entire fee:
        const non_zero_byte_fee = 3n * 16n;
        const zero_byte_fee = 3165n * 4n;
        const base_ethereum_fee = 21000n;
        const modexp_min_cost = 200n * 20n; // see MIN_GAS_COST in frontier's modexp precompile
        const entire_fee = non_zero_byte_fee + zero_byte_fee + base_ethereum_fee + modexp_min_cost;
        console.log("ENTIRE FEE: ", entire_fee);
        // Given that the pov is refunded, the gas used should be the minimum of the legacy gas 
        // and the pov gas.
        const expected = BigInt(Math.min(Number(entire_fee), 3797 * GAS_LIMIT_POV_RATIO));
        expect(receipt.gasUsed, "gasUsed does not match manual calculation").toBe(expected);
      },
    });
  },
});
