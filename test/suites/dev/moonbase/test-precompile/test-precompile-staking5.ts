import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  MIN_GLMR_STAKING,
  ethan,
} from "@moonwall/util";

describeSuite({
  id: "D012885",
  title: "Precompiles - Staking - Join Delegators",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      await context.writePrecompile!({
        precompileName: "ParachainStaking",
        functionName: "delegateWithAutoCompound",
        args: [ALITH_ADDRESS, MIN_GLMR_STAKING, 0, 0, 0, 0],
        privateKey: ETHAN_PRIVATE_KEY,
      });
      await context.createBlock();

      expect(
        await context.readPrecompile!({
          precompileName: "ParachainStaking",
          functionName: "delegationRequestIsPending",
          args: [ETHAN_ADDRESS, ALITH_ADDRESS],
        })
      ).toBe(false);
    });

    it({
      id: "T01",
      title: "should verify delegation pending requests",
      test: async function () {
        // Schedule Revoke
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.scheduleRevokeDelegation(ALITH_ADDRESS)
            .signAsync(ethan)
        );

        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "delegationRequestIsPending",
            args: [ETHAN_ADDRESS, ALITH_ADDRESS],
          })
        ).toBe(true);
      },
    });
  },
});
