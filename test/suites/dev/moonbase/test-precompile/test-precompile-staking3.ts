import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ETHAN_ADDRESS, ETHAN_PRIVATE_KEY, MIN_GLMR_STAKING } from "@moonwall/util";

describeSuite({
  id: "D022870",
  title: "Precompiles - Staking - Collator Leaving",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      const rawTxn = await context.writePrecompile!({
        precompileName: "ParachainStaking",
        functionName: "joinCandidates",
        args: [MIN_GLMR_STAKING, 1],
        privateKey: ETHAN_PRIVATE_KEY,
        rawTxOnly: true,
      });

      const { result } = await context.createBlock(rawTxn);
      const receipt = await context
        .viem()
        .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
      expect(receipt.status).to.equal("success");
    });

    it({
      id: "T01",
      title: "should successfully call candidate_exit_is_pending on goliath",
      test: async function () {
        await context.writePrecompile!({
          precompileName: "ParachainStaking",
          functionName: "scheduleLeaveCandidates",
          args: [2],
          privateKey: ETHAN_PRIVATE_KEY,
        });
        await context.createBlock();

        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "candidateExitIsPending",
            args: [ETHAN_ADDRESS],
          })
        ).toBe(true);
      },
    });
  },
});
