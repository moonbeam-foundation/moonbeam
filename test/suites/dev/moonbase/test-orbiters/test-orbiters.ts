import { describeSuite, expect } from "@moonwall/cli";
import "@moonbeam-network/api-augment";
import { alith, baltathar, charleth, dorothy, ethan, faith, goliath } from "@moonwall/util";
import { jumpRounds } from "../../../../helpers";

describeSuite({
  id: "D012501",
  title: "Orbiters",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    it({
      id: "T01",
      title: "Marking orbiters offline is a noop",
      test: async function () {
        const minCandidateStk = context.polkadotJs().consts.parachainStaking.minCandidateStk;
        await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
              .signAsync(alith),
          ],
          { allowFailures: false }
        );

        await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.enableMarkingOffline(true))
              .signAsync(alith),
          ],
          { allowFailures: false }
        );

        // ceil(2 / 3 * 8) = 6 collators are needed to be able to mark
        // collators offline. Alith is already a collator so 5 extra
        // are added.
        const collators = [baltathar, charleth, dorothy, ethan, faith];

        const joinCandidateTxs = collators.map((c, i) =>
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(minCandidateStk, 1 + i)
            .signAsync(c)
        );
        await context.createBlock(joinCandidateTxs, { allowFailures: false });

        const orbiterPool = collators[1];
        const orbiter = goliath;

        await context.createBlock(
          [
            context
              .polkadotJs()
              .tx.sudo.sudo(
                context.polkadotJs().tx.moonbeamOrbiters.addCollator(orbiterPool.address)
              )
              .signAsync(alith),
          ],
          { allowFailures: false }
        );

        await context.createBlock(
          context.polkadotJs().tx.moonbeamOrbiters.orbiterRegister().signAsync(orbiter),
          { allowFailures: false }
        );

        await context.createBlock(
          context
            .polkadotJs()
            .tx.moonbeamOrbiters.collatorAddOrbiter(orbiter.address)
            .signAsync(orbiterPool),
          { allowFailures: false }
        );

        // Advance some rounds so orbiter is set to active and it goes
        // some rounds without producing blocks
        await jumpRounds(context, 6);

        const afterOrbiterActiveRound = await context.polkadotJs().query.parachainStaking.round();
        const afterOrbiterActiveOrbiter = await context
          .polkadotJs()
          .query.moonbeamOrbiters.orbiterPerRound(
            afterOrbiterActiveRound.current,
            orbiterPool.address
          );
        expect(afterOrbiterActiveOrbiter.toString()).toEqual(orbiter.address);

        const notifyInactiveOrbiter = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.notifyInactiveCollator(orbiter.address)
            .signAsync(alith),
          { allowFailures: true }
        );

        expect(notifyInactiveOrbiter.result!.successful).toEqual(false);
        expect(notifyInactiveOrbiter.result!.error!.name).toEqual("CannotBeNotifiedAsInactive");

        // Call to mark an orbiterPool that has not produced blocks offline
        // should succeed but it should be a noop (pool will still be active) if there is an
        // active orbiter in the pool
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.notifyInactiveCollator(orbiterPool.address)
            .signAsync(alith),
          { allowFailures: false }
        );

        const afterNoopNotifyInfo = await context
          .polkadotJs()
          .query.parachainStaking.candidateInfo(orbiterPool.address);
        expect(afterNoopNotifyInfo.unwrap().status.isActive).toBe(true);

        await context.createBlock(
          context
            .polkadotJs()
            .tx.moonbeamOrbiters.collatorRemoveOrbiter(orbiter.address)
            .signAsync(orbiterPool),
          { allowFailures: false }
        );

        // Advance rounds so that the orbiter gets rotated out of the pool
        // and the OrbiterPerRoundEntry is cleared
        await jumpRounds(context, 6);

        const afterRemoveOrbiterRound = await context.polkadotJs().query.parachainStaking.round();
        const afterRemoveOrbiterOrbiter = await context
          .polkadotJs()
          .query.moonbeamOrbiters.orbiterPerRound(
            afterRemoveOrbiterRound.current,
            orbiterPool.address
          );
        expect(afterRemoveOrbiterOrbiter.isNone).toBe(true);

        // Marking the orbiter pool without active orbiters should
        // make the orbiter pool idle
        await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.notifyInactiveCollator(orbiterPool.address)
            .signAsync(alith),
          { allowFailures: false }
        );

        const afterOrbPoolInnactiveInfo = await context
          .polkadotJs()
          .query.parachainStaking.candidateInfo(orbiterPool.address);
        expect(afterOrbPoolInnactiveInfo.unwrap().status.isIdle).toBe(true);
        const afterOrbPoolInnactiveCandidatePool = await context
          .polkadotJs()
          .query.parachainStaking.candidatePool();
        const afterOrbPoolInnactiveCandidates = afterOrbPoolInnactiveCandidatePool
          .toJSON()
          .map((c) => c.owner);
        expect(afterOrbPoolInnactiveCandidates).not.toContain(orbiterPool.address);
      },
    });
  },
});
