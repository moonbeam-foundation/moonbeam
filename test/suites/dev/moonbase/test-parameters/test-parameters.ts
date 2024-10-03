import { describeSuite, DevModeContext, expect } from "@moonwall/cli";
import "@moonbeam-network/api-augment";
import { alith } from "@moonwall/util";
import { fail } from "assert";
const UNIT = 1_000_000_000_000_000_000n;

const RUNTIME = "MoonbaseRuntime";
const CRATE = "RuntimeParams";
const ALL_PARAMS = "DynamicParams";

function parameterType(context: DevModeContext, module: string, name: string, value: unknown) {
  const paramWrapper = context
    .polkadotJs()
    .createType(`${RUNTIME}${CRATE}${ALL_PARAMS}${module}Parameters`, {
      [name]: [null, value],
    });

  const runtimeParameter = context.polkadotJs().createType(`${RUNTIME}${CRATE}RuntimeParameters`, {
    [module]: paramWrapper,
  });

  return runtimeParameter;
}

function parameterKey(context: DevModeContext, module: string, name: string) {
  const key = context
    .polkadotJs()
    .createType(`${RUNTIME}${CRATE}${ALL_PARAMS}${module}ParametersKey`, {
      [name]: null,
    });

  const keyWrapper = context.polkadotJs().createType(`${RUNTIME}${CRATE}RuntimeParametersKey`, {
    [module]: key,
  });

  return keyWrapper;
}

describeSuite({
  id: "DTemp01",
  title: "Parameters",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let testCounter = 0;
    function testParam(module: string, name: string, valueCreation: [string, unknown]) {
      it({
        id: `T${testCounter++} - ${module} - ${name}`,
        title: "Parameters cannot be changed by normal user",
        test: async () => {
          const value = context.polkadotJs().createType(valueCreation[0], valueCreation[1]);
          const param = parameterType(context, module, name, value);

          const res = await context.createBlock(
            context.polkadotJs().tx.parameters.setParameter(param.toU8a()).signAsync(alith),
            { allowFailures: true }
          );
          expect(res.result?.error?.name).toEqual("BadOrigin");
        },
      });

      it({
        id: `T${testCounter++} - ${module} - ${name}`,
        title: "Parameters can be changed by root user",
        test: async () => {
          const value = context.polkadotJs().createType(valueCreation[0], valueCreation[1]);
          const param = parameterType(context, module, name, value);

          await context.createBlock(
            context
              .polkadotJs()
              .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param.toU8a()))
              .signAsync(alith),
            { allowFailures: false }
          );

          const key = parameterKey(context, module, name);

          const wrappedValue = await context.polkadotJs().query.parameters.parameters(key.toU8a());
          const gotValue = wrappedValue.value.value.value.toU8a();
          expect(gotValue).toEqual(value.toU8a());
        },
      });
    }

    testParam("RuntimeConfig", "FeesTreasuryProportion", ["Perbill", 200_000_000]);
    testParam("PalletRandomness", "Deposit", ["u128", UNIT * 100n]);

    it({
      id: `T${testCounter++} - PalletRandomness - Deposit - CustomTests`,
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
          fail("An extrinsic should not be created, since the parameter is invalid");
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
