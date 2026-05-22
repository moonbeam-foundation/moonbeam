import "@moonbeam-network/api-augment";
import {
  MoonwallContext,
  alith,
  beforeAll,
  type ChopsticksContext,
  describeSuite,
  expect,
} from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import type { FrameSystemEventRecord, SpRuntimeDispatchError } from "@polkadot/types/lookup";
import type { u32 } from "@polkadot/types";
import type { HexString } from "@polkadot/util/types";
import { hexToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import { existsSync, readFileSync } from "node:fs";

const hash = (prefix: HexString, suffix: Uint8Array) =>
  u8aToHex(u8aConcat(hexToU8a(prefix), xxhashAsU8a(suffix, 64), suffix));

const upgradeRestrictionSignal = (paraId: u32) => {
  const prefix = "0xcd710b30bd2eab0352ddcc26417aa194f27bbb460270642b5bcaf032ea04d56a";
  return hash(prefix, paraId.toU8a());
};

// Apply the branch runtime (pre-staged at rtUpgradePath) onto the forked chain.
// Mirrors suites/chopsticks/test-upgrade-chain.ts.
const upgradeRuntime = async (context: ChopsticksContext) => {
  const path = (await MoonwallContext.getContext()).rtUpgradePath;
  if (!path || !existsSync(path)) {
    throw new Error(`Runtime wasm not found at path: ${path}`);
  }
  const rtHex = `0x${readFileSync(path).toString("hex")}`;
  const rtHash = blake2AsHex(rtHex);
  const api = context.polkadotJs();

  await context.setStorage({
    module: "system",
    method: "authorizedUpgrade",
    methodParams: `${rtHash}01`, // 01 => RT version check = true
  });
  await context.createBlock();
  await api.tx.system.applyAuthorizedUpgrade(rtHex).signAndSend(context.keyring.alice);
  const paraId: u32 = await api.query.parachainInfo.parachainId();
  await api.rpc("dev_newBlock", {
    count: 3,
    relayChainStateOverrides: [[upgradeRestrictionSignal(paraId), null]],
  });
};

// Verifies the permissionless `crowdloanRewards.completeUnclaimedRewards` extrinsic against
// real forked state. Runs under the chopsticks `upgrade_*` envs. Chopsticks cannot build
// blocks on forked moonbeam (Polkadot async-backing v2 `InvalidNumberOfDescendants`), so the
// suite skips that chain and runs on moonriver / moonbase; it also skips gracefully if the
// forked chain has no account with outstanding rewards.
describeSuite({
  id: "C02",
  title: "Chopsticks - Crowdloan complete_unclaimed_rewards",
  foundationMethods: "chopsticks",
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;
    let skip = false;
    let target: string;
    let owed: bigint;

    const findOutstanding = async () => {
      const keys = await api.query.crowdloanRewards.accountsPayable.keysPaged({
        args: [],
        pageSize: 50,
      });
      for (const key of keys) {
        const candidate = key.args[0].toString();
        const info = (await api.query.crowdloanRewards.accountsPayable(candidate)).unwrapOr(null);
        if (!info) continue;
        const outstanding = info.totalReward.toBigInt() - info.claimedReward.toBigInt();
        if (outstanding > 0n) return { candidate, outstanding };
      }
      return null;
    };

    // Submit `completeUnclaimedRewards(target)` from Alith and build blocks (via moonwall's
    // createBlock, which builds deterministically from the local pool) until it is included.
    // Returns the events scoped to our extrinsic. `allowFailures` keeps moonwall from throwing
    // on the intentionally-failing second call (T02), so the test can inspect the failure itself.
    const settleAndCollectEvents = async (): Promise<FrameSystemEventRecord[]> => {
      const signed = await api.tx.crowdloanRewards
        .completeUnclaimedRewards(target)
        .signAsync(alith, { nonce: -1 });
      const myHash = signed.hash.toHex();
      await api.rpc.author.submitExtrinsic(signed.toHex());

      for (let attempt = 0; attempt < 3; attempt++) {
        await context.createBlock({ allowFailures: true });
        const block = await api.rpc.chain.getBlock();
        const idx = block.block.extrinsics.findIndex((ex) => ex.hash.toHex() === myHash);
        if (idx >= 0) {
          const apiAt = await api.at(block.block.header.hash);
          const events = await apiAt.query.system.events();
          return events.filter(
            ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.toNumber() === idx
          );
        }
      }
      throw new Error("completeUnclaimedRewards extrinsic was not included in a block");
    };

    beforeAll(async () => {
      api = context.polkadotJs();
      const specName = api.consts.system.version.specName.toString();

      // Chopsticks can't build blocks on forked moonbeam; don't attempt the upgrade there.
      if (specName === "moonbeam") {
        skip = true;
        log("Skipping on moonbeam: chopsticks cannot build blocks (InvalidNumberOfDescendants)");
        return;
      }

      await upgradeRuntime(context);
      expect(
        api.tx.crowdloanRewards.completeUnclaimedRewards !== undefined,
        "completeUnclaimedRewards must exist after the upgrade"
      ).to.be.true;

      // Let multi-block migrations triggered by the upgrade drain before dispatching.
      for (let i = 0; i < 20; i++) {
        const events = await api.query.system.events();
        if (
          events.some(({ event }) => api.events.multiBlockMigrations.UpgradeCompleted.is(event))
        ) {
          break;
        }
        const started = events.some(({ event }) =>
          api.events.multiBlockMigrations.UpgradeStarted.is(event)
        );
        if (!started && i > 0) break;
        await context.createBlock();
      }

      const found = await findOutstanding();
      if (!found) {
        skip = true;
        log(`Skipping on ${specName}: no account with outstanding crowdloan rewards`);
        return;
      }
      target = found.candidate;
      owed = found.outstanding;
      log(`Target ${target} has ${owed} outstanding (will be settled by Alith)`);
    });

    it({
      id: "T01",
      title: "drains the full outstanding reward to the target, free for the caller",
      test: async () => {
        if (skip) return;

        const targetBefore = (await api.query.system.account(target)).data.free.toBigInt();
        const callerBefore = (await api.query.system.account(alith.address)).data.free.toBigInt();
        expect(alith.address.toLowerCase()).to.not.equal(target.toLowerCase());

        const ours = await settleAndCollectEvents();

        const rewardsPaid = ours.find(({ event }) =>
          api.events.crowdloanRewards.RewardsPaid.is(event)
        );
        expect(rewardsPaid, "RewardsPaid event should be emitted").toBeDefined();
        expect((rewardsPaid!.event.data[0] as any).toString().toLowerCase()).to.equal(
          target.toLowerCase()
        );
        expect((rewardsPaid!.event.data[1] as any).toBigInt()).to.equal(owed);

        const success = ours.find(({ event }) => api.events.system.ExtrinsicSuccess.is(event));
        expect(success, "ExtrinsicSuccess should be emitted").toBeDefined();
        expect((success!.event.data[0] as any).paysFee.toString()).to.equal("No");

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
      test: async () => {
        if (skip) return;

        const ours = await settleAndCollectEvents();
        const failed = ours.find(({ event }) => api.events.system.ExtrinsicFailed.is(event));
        expect(failed, "second call should fail").toBeDefined();

        const dispatchError = failed!.event.data[0] as SpRuntimeDispatchError;
        expect(dispatchError.isModule).to.be.true;
        const decoded = api.registry.findMetaError(dispatchError.asModule);
        expect(`${decoded.section}.${decoded.method}`).to.equal(
          "crowdloanRewards.NoAssociatedClaim"
        );
        expect((failed!.event.data[1] as any).paysFee.toString()).to.equal("Yes");
      },
    });
  },
});
