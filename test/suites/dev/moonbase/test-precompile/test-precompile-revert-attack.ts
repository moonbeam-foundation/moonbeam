import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, MIN_GLMR_STAKING } from "@moonwall/util";

//     // We delegate our target collator with all the tokens provided
//     staking.delegate(target, msg.value);
//     revert("By reverting this transaction, we return the eth to the caller");
// }
// Would the delegation pass in substrate but get the eth back in the evm?
// We have to make sure that's not possible

describeSuite({
  id: "D012873",
  title: "Precompiles - Reverting Staking precompile",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    it({
      id: "T01",
      title: "should not revert the whole transaction cost",
      test: async function () {
        const initialBalance = await context.viem().getBalance({ address: ALITH_ADDRESS });

        const { contractAddress } = await context.deployContract!("StakingAttacker");

        const rawTxn = await context.writeContract!({
          contractAddress,
          contractName: "StakingAttacker",
          functionName: "score_a_free_delegation",
          value: MIN_GLMR_STAKING,
          gas: 5_000_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock!(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expect(receipt.status).to.eq("reverted");

        const nominatorsAfter = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(ALITH_ADDRESS);
        expect(nominatorsAfter.isEmpty, "Delegation shouldn't have passed").toBe(true);

        expect(
          initialBalance - (await context.viem().getBalance({ address: ALITH_ADDRESS })),
          "balance dif should only be tx fee, not MIN_GLMR_STAKING"
        ).toBeLessThan(MIN_GLMR_STAKING);

        expect(
          async () =>
            await context.writeContract!({
              contractAddress,
              contractName: "StakingAttacker",
              functionName: "score_a_free_delegation",
              value: MIN_GLMR_STAKING,
            })
        ).rejects.toThrowError(
          "Module(ModuleError { index: 12, error: [10, 0, 0, 0], " +
            'message: Some("DelegationBelowMin") })'
        );
      },
    });
  },
});
