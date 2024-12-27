import "@moonbeam-network/api-augment";
import type { ApiDecoration } from "@polkadot/api/types";
import type { bool, Option, u32 } from "@polkadot/types-codec";
import type {
  FrameSystemEventRecord,
  PalletMoonbeamOrbitersCollatorPoolInfo,
} from "@polkadot/types/lookup";
import type { AccountId20 } from "@polkadot/types/interfaces";
import { sortObjectByKeys } from "../../helpers/common.js";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import type { StorageKey } from "@polkadot/types";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S15",
  title: "Verify orbiters",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber = 0;
    let apiAt: ApiDecoration<"promise">;
    let collatorsPools: [
      StorageKey<[AccountId20]>,
      Option<PalletMoonbeamOrbitersCollatorPoolInfo>,
    ][];
    let registeredOrbiters: [StorageKey<[AccountId20]>, Option<bool>][];
    let counterForCollatorsPool: u32;
    let currentRound: number;
    let orbiterPerRound: [StorageKey<[u32, AccountId20]>, Option<AccountId20>][];
    let events: FrameSystemEventRecord[];
    let specVersion = 0;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const runtimeVersion = paraApi.runtimeVersion.specVersion.toNumber();
      atBlockNumber = process.env.BLOCK_NUMBER
        ? Number.parseInt(process.env.BLOCK_NUMBER)
        : (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
      collatorsPools = await apiAt.query.moonbeamOrbiters.collatorsPool.entries();
      registeredOrbiters =
        runtimeVersion >= 1605
          ? await apiAt.query.moonbeamOrbiters.registeredOrbiter.entries()
          : [];
      counterForCollatorsPool = await apiAt.query.moonbeamOrbiters.counterForCollatorsPool();
      currentRound = (await apiAt.query.parachainStaking.round()).current.toNumber();
      orbiterPerRound = await apiAt.query.moonbeamOrbiters.orbiterPerRound.entries();
      events = await apiAt.query.system.events();
      specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
    });

    it({
      id: "C100",
      title: "should have reserved tokens",
      test: async function () {
        const reserves = await apiAt.query.balances.reserves.entries();
        const orbiterReserves = reserves
          .map((reserveSet) =>
            reserveSet[1].find((r) => r.id.toUtf8() === "orbi")
              ? `0x${reserveSet[0].toHex().slice(-40)}`
              : null
          )
          .filter((r) => !!r);

        const orbiterRegisteredAccounts = registeredOrbiters.map(
          (o) => `0x${o[0].toHex().slice(-40)}`
        );

        for (const reservedAccount of orbiterReserves) {
          expect(
            orbiterRegisteredAccounts,
            `Account ${reservedAccount} has "orbi" reserve but is not orbiter.`
          ).to.include(reservedAccount);
        }

        for (const orbiterAccount of orbiterRegisteredAccounts) {
          expect(
            orbiterReserves,
            `Account ${orbiterAccount} is orbiter but doesn't have "orbi" reserve.`
          ).to.include(orbiterAccount);
        }
        log(`Verified ${orbiterRegisteredAccounts.length} orbiter reserves`);
      },
    });

    it({
      id: "C200",
      title: "should be registered if in a pool",
      test: async function () {
        for (const orbiterPool of collatorsPools) {
          const collator = `0x${orbiterPool[0].toHex().slice(-40)}`;
          const pool = orbiterPool[1].unwrap();
          const orbiterRegisteredAccounts = registeredOrbiters.map(
            (o) => `0x${o[0].toHex().slice(-40)}`
          );
          if (pool.maybeCurrentOrbiter.isSome) {
            const selectedOrbiter = pool.maybeCurrentOrbiter.unwrap().accountId.toHex();
            const isRemoved = pool.maybeCurrentOrbiter.unwrap().removed.isTrue;
            const poolOrbiters = pool.orbiters.map((o) => o.toHex());

            if (isRemoved) {
              expect(
                poolOrbiters,
                `Selected orbiter ${selectedOrbiter} is removed but ` +
                  `still in the pool ${collator} orbiters`
              ).to.not.include(selectedOrbiter);
            } else {
              expect(
                poolOrbiters,
                `Selected orbiter ${selectedOrbiter} is not in the pool ${collator} orbiters`
              ).to.include(selectedOrbiter);
            }

            expect(
              orbiterRegisteredAccounts,
              `Account ${selectedOrbiter} is in a pool but not registered`
            ).to.include(selectedOrbiter);
          }
        }

        log(`Verified ${collatorsPools.length} orbiter pools`);
      },
    });

    it({
      id: "C300",
      title: "should not have more pool than the max allowed",
      test: async function () {
        expect(collatorsPools.length, `Orbiter pool is too big`).to.be.at.most(
          counterForCollatorsPool.toNumber()
        );

        log(`Verified orbiter pools size`);
      },
    });

    it({
      id: "C400",
      title: "should have matching rewards",
      test: async function () {
        if (specVersion >= 1800) {
          const rotatePeriod: number = (
            (await apiAt.consts.moonbeamOrbiters.rotatePeriod) as any
          ).toNumber();

          // Get parent collators
          const parentCollators = new Set();
          collatorsPools.forEach((o) => parentCollators.add(o[0].args[0].toHex()));

          // Get collators rewards
          const collatorRewards = {};
          for (const { event, phase } of events) {
            if (
              phase.isInitialization &&
              event.section === "parachainStaking" &&
              event.method === "Rewarded"
            ) {
              const data = event.data as any;
              const account = data.account.toHex();
              const rewards = data.rewards.toBigInt();
              if (parentCollators.has(account)) {
                collatorRewards[account] = rewards;
              }
            }
          }

          //console.log(collatorRewards);

          if (Object.keys(collatorRewards).length > 0) {
            // Compute expected reward for each orbiter
            const lastRotateRound = currentRound - (currentRound % rotatePeriod);
            const expectedOrbiterRewards = {};
            orbiterPerRound.forEach((o) => {
              const [round, collator] = o[0].args;
              const orbiter = o[1];

              if (round.toNumber() === lastRotateRound && collatorRewards[collator.toHex()]) {
                expectedOrbiterRewards[orbiter.unwrap().toHex()] =
                  collatorRewards[collator.toHex()];
              }
            });
            const sortedExpectedOrbiterRewards = sortObjectByKeys(expectedOrbiterRewards);

            // Verify orbiters rewards
            const actualOrbiterRewards = {};
            for (const { event, phase } of events) {
              if (
                phase.isInitialization &&
                event.section === "MoonbeamOrbiters" &&
                event.method === "OrbiterRewarded"
              ) {
                const data = event.data as any;
                const orbiter = data.account.toHex();
                const rewards = data.rewards.toBigInt();
                actualOrbiterRewards[orbiter] = rewards;
              }
            }
            const sortedActualOrbiterRewards = sortObjectByKeys(actualOrbiterRewards);

            //console.log(sortedExpectedOrbiterRewards);
            //console.log(sortedActualOrbiterRewards);

            expect(
              sortedActualOrbiterRewards,
              `Orbiter rewards doesn't match expectation for block #${atBlockNumber}.`
            ).to.deep.equal(sortedExpectedOrbiterRewards);
          }
        }
      },
    });
  },
});
