import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  GLMR,
  MIN_GLMR_DELEGATOR,
  alith,
  createRawTransfer,
  generateKeyringPair,
} from "@moonwall/util";
import { createProposal } from "../../../../helpers";

describeSuite({
  id: "D021801",
  title: "Locks - Voting and staking locks are not mutually exclusive",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let proposalIndex: number;
    const randomAccount = generateKeyringPair();
    const randomAddress = randomAccount.address as `0x${string}`;
    beforeAll(async function () {
      await context.createBlock(
        createRawTransfer(context, randomAddress, MIN_GLMR_DELEGATOR + GLMR)
      );
      proposalIndex = await createProposal({ context });
    });

    it({
      id: "T01",
      title: "should be able to vote (conviction) and stake",
      test: async function () {
        // Vote yes on proposal with 1 GLMR
        await context.createBlock(
          context
            .polkadotJs()
            .tx.convictionVoting.vote(proposalIndex, {
              Standard: { vote: { aye: true, conviction: "Locked5x" }, balance: GLMR },
            })
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        // Delegate to Alith with MIN_GLMR_DELEGATOR
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              0,
              0,
              0
            )
            .signAsync(randomAccount),
          { allowFailures: false }
        );

        // check system balance
        const frozenBalance = (
          await context.polkadotJs().query.system.account(randomAddress)
        ).data.frozen.toBigInt();

        // BigInt doesn't have max()- we are testing frozenBalance === max(GLMR, MIN_GLMR_DELEGATOR)
        if (GLMR > MIN_GLMR_DELEGATOR) {
          expect(frozenBalance).to.equal(GLMR);
        } else {
          expect(frozenBalance).to.equal(MIN_GLMR_DELEGATOR);
        }

        // check locked balances
        const lockedBalances = await context.polkadotJs().query.balances.locks(randomAddress);
        expect(lockedBalances.length).to.equal(2);
        expect(lockedBalances[0].amount.toBigInt()).to.equal(GLMR);
        expect(lockedBalances[1].amount.toBigInt()).to.equal(MIN_GLMR_DELEGATOR);
      },
    });
  },
});
