import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";
import { jumpToRound } from "../../../../helpers";

describeSuite({
  id: "D023490",
  title: "Staking - Delegation Scheduled Requests - multiple bond less and revoke interactions",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const LESS_AMOUNT = 10n;

    beforeAll(async () => {
      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
          .signAsync(alith)
      );
    });

    it({
      id: "T01",
      title: "can schedule up to 50 bond_less requests and execute them in order",
      test: async () => {
        const api = context.polkadotJs();
        const psTx = api.tx.parachainStaking;
        const psQuery = api.query.parachainStaking;

        // Ensure a fresh delegation with enough stake for many decreases.
        await context.createBlock(
          psTx
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR + 1000n, 0, 0, 0, 0)
            .signAsync(ethan),
          { allowFailures: false }
        );

        const NUM_REQUESTS = 50;
        for (let i = 0; i < NUM_REQUESTS; i++) {
          await context.createBlock(
            psTx.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT).signAsync(ethan),
            { allowFailures: false }
          );
        }

        const scheduled = await psQuery.delegationScheduledRequests(alith.address, ethan.address);
        expect(scheduled.toJSON()).to.have.length(NUM_REQUESTS);

        const firstWhenExecutable = scheduled[0].whenExecutable.toNumber();
        await jumpToRound(context, firstWhenExecutable);

        for (let i = 0; i < NUM_REQUESTS; i++) {
          await context.createBlock(
            psTx.executeDelegationRequest(ethan.address, alith.address).signAsync(ethan),
            { allowFailures: false }
          );
        }

        const remaining = await psQuery.delegationScheduledRequests(alith.address, ethan.address);
        expect(remaining.toJSON()).to.deep.equal([]);
      },
    });

    it({
      id: "T02",
      title: "cannot schedule bond_less when a revoke request is already scheduled",
      test: async () => {
        const api = context.polkadotJs();
        const psTx = api.tx.parachainStaking;

        await context.createBlock(psTx.scheduleRevokeDelegation(alith.address).signAsync(ethan), {
          allowFailures: false,
        });

        const block = await context.createBlock(
          psTx.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT).signAsync(ethan),
          { allowFailures: true }
        );
        expect(block.result!.error!.name).to.equal("PendingDelegationRequestAlreadyExists");
      },
    });

    it({
      id: "T03",
      title: "cannot schedule revoke when bond_less requests are already scheduled",
      test: async () => {
        const api = context.polkadotJs();
        const psTx = api.tx.parachainStaking;

        // Ensure there is no pending request for this delegation.
        await context.createBlock(psTx.cancelDelegationRequest(alith.address).signAsync(ethan), {
          allowFailures: true,
        });

        await context.createBlock(
          psTx.scheduleDelegatorBondLess(alith.address, LESS_AMOUNT).signAsync(ethan),
          { allowFailures: false }
        );

        const block = await context.createBlock(
          psTx.scheduleRevokeDelegation(alith.address).signAsync(ethan),
          { allowFailures: true }
        );
        expect(block.result!.error!.name).to.equal("PendingDelegationRequestAlreadyExists");
      },
    });
  },
});
