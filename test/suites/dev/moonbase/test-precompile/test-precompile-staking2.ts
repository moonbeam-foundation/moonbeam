import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ETHAN_ADDRESS, ETHAN_PRIVATE_KEY, MIN_GLMR_STAKING } from "@moonwall/util";
import { verifyLatestBlockFees } from "../../../../helpers";

describeSuite({
  id: "D012984",
  title: "Precompiles - Staking - Join Candidates",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      const rawTxn = await context.writePrecompile!({
        precompileName: "ParachainStaking",
        functionName: "joinCandidates",
        args: [MIN_GLMR_STAKING, 1],
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
      title: "should successfully call joinCandidates on ethan",
      test: async function () {
        const candidatesAfter = await context.polkadotJs().query.parachainStaking.candidatePool();
        expect(candidatesAfter.length).to.equal(2, "New candidate should have been added");
        expect(
          candidatesAfter[1].owner.toString(),
          "New candidate ethan should have been added"
        ).toBe(ETHAN_ADDRESS);
        expect(candidatesAfter[1].amount.toBigInt()).to.equal(
          1000000000000000000000n,
          "new candidate ethan should have been added (wrong amount)"
        );

        expect(
          await context.readPrecompile!({
            precompileName: "ParachainStaking",
            functionName: "isCandidate",
            args: [ALITH_ADDRESS],
          })
        ).toBe(true);

        await verifyLatestBlockFees(context, 0n);
      },
    });
  },
});
