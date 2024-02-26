import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ETHAN_ADDRESS, ETHAN_PRIVATE_KEY, MIN_GLMR_STAKING } from "@moonwall/util";

describeSuite({
  id: "D012986",
  title: "Precompiles - Staking - Join Delegators",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      const rawTxn = await context.writePrecompile!({
        precompileName: "ParachainStaking",
        functionName: "delegate",
        args: [ALITH_ADDRESS, MIN_GLMR_STAKING, 0, 0],
        rawTxOnly: true,
        privateKey: ETHAN_PRIVATE_KEY,
      });

      const { result } = await context.createBlock(rawTxn);
      const receipt = await context
        .viem()
        .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
      expect(receipt.status).to.equal("success");
    });

    it({
      id: "T01",
      title: "should have successfully delegated ALITH",
      test: async function () {
        const delegatorsAfter = (
          await context.polkadotJs().query.parachainStaking.delegatorState(ETHAN_ADDRESS)
        ).unwrap();

        expect(delegatorsAfter.delegations[0].owner.toString(), "Delegation unsucessful").to.equal(
          ALITH_ADDRESS
        );
        expect(delegatorsAfter.status.toString()).equal("Active");
      },
    });

    it({
      id: "T02",
      title: "should have correct delegation amount for ethan to ALITH",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "delegationAmount",
            args: [ETHAN_ADDRESS, ALITH_ADDRESS],
          })
        ).toBe(MIN_GLMR_STAKING);
      },
    });

    it({
      id: "T03",
      title: "should have 0 delegation amount for non-existent delegation",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "delegationAmount",
            args: [ALITH_ADDRESS, ALITH_ADDRESS],
          })
        ).toBe(0n);
      },
    });

    it({
      id: "T04",
      title: "should have ethan's delegation to ALITH in top delegations",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "isInTopDelegations",
            args: [ETHAN_ADDRESS, ALITH_ADDRESS],
          })
        ).toBe(true);
      },
    });

    it({
      id: "T05",
      title: "should not be in top delegations when non-existent delegation ",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "isInTopDelegations",
            args: [ALITH_ADDRESS, ALITH_ADDRESS],
          })
        ).toBe(false);
      },
    });
  },
});
