import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { createViemTransaction } from "@moonwall/util";
import { ConstantStore } from "../../../../helpers/constants";
import { hexToU8a } from "@polkadot/util";
import { calculateEIP7623Gas } from "../../../../helpers/fees";

describeSuite({
  id: "D021607",
  title: "Substrate Length Fees - Ethereum txn Interaction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should not charge length fee for precompile from Ethereum txn",
      test: async () => {
        const { specVersion } = await context.polkadotJs().consts.system.version;
        const constants = ConstantStore(context);
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

        const inputData =
          "0x0000000000000000000000000000000000000000000000000000000000000004" + // base
          "0000000000000000000000000000000000000000000000000000000000000004" + // exp
          "0000000000000000000000000000000000000000000000000000000000000004" + // mod
          "0".repeat(2048) + // 2048 hex nibbles -> 1024 bytes
          "0".repeat(2048) +
          "0".repeat(2048);

        const tx = await createViemTransaction(context, {
          to: MODEXP_PRECOMPILE_ADDRESS,
          gas: BigInt(constants.EXTRINSIC_GAS_LIMIT.get(specVersion.toNumber())),
          data: inputData as `0x${string}`,
        });

        const { result } = await context.createBlock(tx);
        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).toBe("success");

        // Calculate byte counts for EIP-7623
        const byteArray = hexToU8a(inputData);
        const numZeroBytes = byteArray.filter((a) => a === 0).length;
        const numNonZeroBytes = byteArray.length - numZeroBytes;

        // rough math on what the exponential LengthToFee modifier would do to this:
        // * input data alone is (3 * 1024) + (3 * 32) = 3168
        // * 3168 ** 3 = 31_794_757_632
        // * 31_794_757_632 / WEIGHT_PER_GAS = 1_271_790
        //
        // conclusion: the LengthToFee modifier is NOT involved
        expect(receipt.gasUsed, "gasUsed does not match manual calculation").toBeLessThan(
          1_271_790n
        );

        // Calculate execution gas costs
        const is_precompile_check_gas = 1669n;
        const modexp_min_cost = 200n * 20n; // see MIN_GAS_COST in frontier's modexp precompile
        const executionGas = modexp_min_cost + is_precompile_check_gas;

        // Calculate expected gas with EIP-7623
        const expectedGasUsed = calculateEIP7623Gas(numZeroBytes, numNonZeroBytes, executionGas);

        // the gas used should be the maximum of the legacy gas and the pov gas
        const expectedWithPov = BigInt(
          Math.max(
            Number(expectedGasUsed),
            3821 * Number(constants.GAS_PER_POV_BYTES.get(specVersion.toNumber()))
          )
        );

        expect(receipt.gasUsed, "gasUsed does not match manual calculation").toBe(expectedWithPov);
      },
    });
  },
});
