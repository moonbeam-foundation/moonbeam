import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { fail } from "assert";
import { parameterType, UNIT } from "./test-parameters";

describeSuite({
  id: "DTemp02",
  title: "Parameters - Pallet Randomness",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    it({
      id: `T01 - PalletRandomness - Deposit - CustomTests`,
      title: "Deposit parameter should only be accepted in bounds",
      test: async () => {
        const MIN = 1n * UNIT;
        const MAX = 1000n * UNIT;

        // used as an acceptable value
        const AVG = (MIN + MAX) / 2n;

        const param1 = parameterType(context, "PalletRandomness", "Deposit", MIN - 1n);
        try {
          await context.createBlock(
            context
              .polkadotJs()
              .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param1.toU8a()))
              .signAsync(alith),
            { allowFailures: false }
          );
          fail("An extrinsic should not be created, since the parameter is invalid");
        } catch (error) {
          expect(error.toString().toLowerCase()).to.contain("value out of bounds");
        }

        const param2 = parameterType(context, "PalletRandomness", "Deposit", MAX + 1n);
        try {
          await context.createBlock(
            context
              .polkadotJs()
              .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param2.toU8a()))
              .signAsync(alith),
            { allowFailures: false }
          );
          expect.fail("An extrinsic should not be created, since the parameter is invalid");
        } catch (error) {
          expect(error.toString().toLowerCase()).to.contain("value out of bounds");
        }

        const param3 = parameterType(context, "PalletRandomness", "Deposit", AVG);
        const res3 = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param3.toU8a()))
            .signAsync(alith),
          { allowFailures: false }
        );
        expect(
          res3.result?.successful,
          "An extrinsic should be created, since the parameter is valid"
        ).to.be.true;
      },
    });
  },
});
