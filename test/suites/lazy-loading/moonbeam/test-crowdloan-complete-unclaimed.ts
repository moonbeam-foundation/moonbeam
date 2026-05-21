import "@moonbeam-network/api-augment";
import { alith, beforeAll, describeSuite, expect } from "moonwall";
import { RUNTIME_CONSTANTS } from "../../../helpers";
import type { ApiPromise } from "@polkadot/api";
import type { SpRuntimeDispatchError } from "@polkadot/types/lookup";
import fs from "node:fs/promises";
import { u8aToHex } from "@polkadot/util";
import assert from "node:assert";

// Verifies the permissionless `crowdloanRewards.completeUnclaimedRewards` extrinsic
// against real Moonbeam mainnet state (lazy loading). The branch runtime is brought
// in via an authorized upgrade (the lazy-loading env pre-authorizes its hash), then
// the test picks a genuine account that still has outstanding (unclaimed) rewards and
// has a *different* account (Alith) settle it in full, checking the lazy-migration
// semantics:
//   - the full outstanding amount is transferred from the crowdloan pot,
//   - the AccountsPayable entry is removed,
//   - the call is free on success (Pays::No),
//   - a second call for the same target fails with NoAssociatedClaim and pays fees.
describeSuite({
  id: "LL-MOONBEAM-CROWDLOAN-COMPLETE",
  title: "Lazy Loading - Crowdloan complete_unclaimed_rewards",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;
    let target: string;
    let owed: bigint;

    beforeAll(async () => {
      api = context.polkadotJs();

      // Bring the fork up to the branch runtime (which carries the new extrinsic) via
      // the upgrade pre-authorized by the lazy-loading env's state overrides.
      const runtimeChain = api.runtimeChain.toUpperCase();
      const runtime = runtimeChain
        .split(" ")
        .filter((v) => Object.keys(RUNTIME_CONSTANTS).includes(v))
        .join()
        .toLowerCase();
      const wasmPath = `../target/release/wbuild/${runtime}-runtime/${runtime}_runtime.compact.compressed.wasm`;
      const runtimeWasmHex = u8aToHex(await fs.readFile(wasmPath));

      const rtBefore = api.consts.system.version.specVersion.toNumber();
      await context.createBlock();
      const { result } = await context.createBlock(
        api.tx.system.applyAuthorizedUpgrade(runtimeWasmHex)
      );
      assert(result, "Block has no extrinsic results");
      const upgradeErrors = result.events
        .filter(({ event }) => api.events.system.ExtrinsicFailed.is(event))
        .map(({ event: { data } }) => {
          const err = data[0] as SpRuntimeDispatchError;
          return err.isModule ? api.registry.findMetaError(err.asModule).method : err.toString();
        });
      if (upgradeErrors.length) {
        throw new Error(`Could not apply runtime upgrade: ${upgradeErrors.join(", ")}`);
      }

      // GoAhead signal, then the block that enacts the new runtime.
      await context.createBlock();
      await context.createBlock();
      const rtAfter = api.consts.system.version.specVersion.toNumber();
      log(`Runtime upgraded from spec ${rtBefore} to ${rtAfter}`);
      expect(rtAfter, "runtime upgrade should raise spec version").to.be.greaterThan(rtBefore);

      // Let any multi-block migrations triggered by the upgrade drain, so they don't
      // compete with our dispatch for block weight.
      for (let i = 0; i < 30; i++) {
        const events = await api.query.system.events();
        const done = events.some(({ event }) =>
          api.events.multiBlockMigrations.UpgradeCompleted.is(event)
        );
        const started = events.some(({ event }) =>
          api.events.multiBlockMigrations.UpgradeStarted.is(event)
        );
        if (done || (!started && i > 0)) break;
        await context.createBlock();
      }

      expect(
        api.tx.crowdloanRewards.completeUnclaimedRewards !== undefined,
        "completeUnclaimedRewards must exist after the upgrade"
      ).to.be.true;

      // Find a real account that still has outstanding rewards. We only scan a small
      // page of keys (enumerating the whole map over lazy loading would be far too many
      // remote reads); a large fraction of entries are outstanding, so a small sample
      // reliably contains one.
      const keys = await api.query.crowdloanRewards.accountsPayable.keysPaged({
        args: [],
        pageSize: 50,
      });
      for (const key of keys) {
        const candidate = key.args[0].toString();
        const info = (await api.query.crowdloanRewards.accountsPayable(candidate)).unwrapOr(null);
        if (!info) continue;
        const outstanding = info.totalReward.toBigInt() - info.claimedReward.toBigInt();
        if (outstanding > 0n) {
          target = candidate;
          owed = outstanding;
          break;
        }
      }
      expect(target, "expected at least one account with outstanding rewards").toBeDefined();
      log(`Target ${target} has ${owed} outstanding (will be settled by Alith)`);
    });

    it({
      id: "T01",
      title: "drains the full outstanding reward to the target, free for the caller",
      test: async function () {
        const targetBefore = (await api.query.system.account(target)).data.free.toBigInt();
        const callerBefore = (await api.query.system.account(alith.address)).data.free.toBigInt();

        // Alith is not the reward target: this is a permissionless settlement.
        expect(alith.address.toLowerCase()).to.not.equal(target.toLowerCase());

        const { result } = await context.createBlock(
          api.tx.crowdloanRewards.completeUnclaimedRewards(target).signAsync(alith)
        );
        const events = result?.events ?? [];

        // RewardsPaid(target, owed) is emitted with the full outstanding amount.
        const rewardsPaid = events.find(({ event }) =>
          api.events.crowdloanRewards.RewardsPaid.is(event)
        );
        expect(rewardsPaid, "RewardsPaid event should be emitted").toBeDefined();
        expect((rewardsPaid!.event.data[0] as any).toString().toLowerCase()).to.equal(
          target.toLowerCase()
        );
        expect((rewardsPaid!.event.data[1] as any).toBigInt()).to.equal(owed);

        // Success path is free for the caller (Pays::No).
        const success = events.find(({ event }) => api.events.system.ExtrinsicSuccess.is(event));
        expect(success, "ExtrinsicSuccess should be emitted").toBeDefined();
        expect((success!.event.data[0] as any).paysFee.toString()).to.equal("No");

        // Target credited the full outstanding amount; entry removed; caller paid nothing.
        const targetAfter = (await api.query.system.account(target)).data.free.toBigInt();
        expect(targetAfter - targetBefore).to.equal(owed);

        const callerAfter = (await api.query.system.account(alith.address)).data.free.toBigInt();
        expect(callerAfter - callerBefore, "caller pays no fee on success").to.equal(0n);

        const entryAfter = (await api.query.crowdloanRewards.accountsPayable(target)).unwrapOr(
          null
        );
        expect(entryAfter, "AccountsPayable entry should be removed").to.be.null;
      },
    });

    it({
      id: "T02",
      title: "second call for the same target fails with NoAssociatedClaim and pays fees",
      test: async function () {
        const { result } = await context.createBlock(
          api.tx.crowdloanRewards.completeUnclaimedRewards(target).signAsync(alith)
        );
        const events = result?.events ?? [];

        const failed = events.find(({ event }) => api.events.system.ExtrinsicFailed.is(event));
        expect(failed, "second call should fail").toBeDefined();

        const dispatchError = failed!.event.data[0] as SpRuntimeDispatchError;
        expect(dispatchError.isModule).to.be.true;
        const decoded = api.registry.findMetaError(dispatchError.asModule);
        expect(`${decoded.section}.${decoded.method}`).to.equal(
          "crowdloanRewards.NoAssociatedClaim"
        );

        // Failure path is not free, so the extrinsic is not DoS-able.
        expect((failed!.event.data[1] as any).paysFee.toString()).to.equal("Yes");
      },
    });
  },
});
