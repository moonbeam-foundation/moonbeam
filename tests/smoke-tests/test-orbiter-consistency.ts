import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { bool, Option, u32 } from "@polkadot/types-codec";
import type {
  FrameSystemEventRecord,
  PalletMoonbeamOrbitersCollatorPoolInfo,
} from "@polkadot/types/lookup";
import type { AccountId20 } from "@polkadot/types/interfaces";
import { expect } from "chai";
import { sortObjectByKeys } from "../util/common";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { StorageKey } from "@polkadot/types";
const debug = require("debug")("smoke:orbiter");

describeSmokeSuite("S1400", `Verify orbiters`, (context, testIt) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let collatorsPools: [
    StorageKey<[AccountId20]>,
    Option<PalletMoonbeamOrbitersCollatorPoolInfo>
  ][] = null;
  let registeredOrbiters: [StorageKey<[AccountId20]>, Option<bool>][] = null;
  let counterForCollatorsPool: u32 = null;
  let currentRound: number = null;
  let orbiterPerRound: [StorageKey<[u32, AccountId20]>, Option<AccountId20>][] = null;
  let events: FrameSystemEventRecord[] = null;
  let specVersion: number = 0;

  before("Setup api & retrieve data", async function () {
    const runtimeVersion = context.polkadotApi.runtimeVersion.specVersion.toNumber();
    atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    collatorsPools = await apiAt.query.moonbeamOrbiters.collatorsPool.entries();
    registeredOrbiters =
      runtimeVersion >= 1605 ? await apiAt.query.moonbeamOrbiters.registeredOrbiter.entries() : [];
    counterForCollatorsPool = await apiAt.query.moonbeamOrbiters.counterForCollatorsPool();
    currentRound = (await apiAt.query.parachainStaking.round()).current.toNumber();
    orbiterPerRound = await apiAt.query.moonbeamOrbiters.orbiterPerRound.entries();
    events = await apiAt.query.system.events();
    specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
  });

  testIt("C100", `should have reserved tokens`, async function () {
    const reserves = await apiAt.query.balances.reserves.entries();
    const orbiterReserves = reserves
      .map((reserveSet) =>
        reserveSet[1].find((r) => r.id.toUtf8() == "orbi")
          ? `0x${reserveSet[0].toHex().slice(-40)}`
          : null
      )
      .filter((r) => !!r);

    const orbiterRegisteredAccounts = registeredOrbiters.map((o) => `0x${o[0].toHex().slice(-40)}`);

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
    debug(`Verified ${orbiterRegisteredAccounts.length} orbiter reserves`);
  });

  testIt("C200", `should be registered if in a pool`, async function () {
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

    debug(`Verified ${collatorsPools.length} orbiter pools`);
  });

  testIt("C300", `should not have more pool than the max allowed`, async function () {
    expect(collatorsPools.length, `Orbiter pool is too big`).to.be.at.most(
      counterForCollatorsPool.toNumber()
    );

    debug(`Verified orbiter pools size`);
  });

  testIt("C400", `should have matching rewards`, async function () {
    if (specVersion >= 1800) {
      let rotatePeriod: number = (
        (await apiAt.consts.moonbeamOrbiters.rotatePeriod) as any
      ).toNumber();

      // Get parent collators
      const parentCollators = new Set();
      collatorsPools.forEach((o) => parentCollators.add(o[0].args[0].toHex()));

      // Get collators rewards
      let collatorRewards = {};
      for (const { event, phase } of events) {
        if (
          phase.isInitialization &&
          event.section == "parachainStaking" &&
          event.method == "Rewarded"
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
        let expectedOrbiterRewards = {};
        orbiterPerRound.forEach((o) => {
          let [round, collator] = o[0].args;
          let orbiter = o[1];

          if (round.toNumber() == lastRotateRound && collatorRewards[collator.toHex()]) {
            expectedOrbiterRewards[orbiter.unwrap().toHex()] = collatorRewards[collator.toHex()];
          }
        });
        const sortedExpectedOrbiterRewards = sortObjectByKeys(expectedOrbiterRewards);

        // Verify orbiters rewards
        let actualOrbiterRewards = {};
        for (const { event, phase } of events) {
          if (
            phase.isInitialization &&
            event.section == "MoonbeamOrbiters" &&
            event.method == "OrbiterRewarded"
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
  });
});
