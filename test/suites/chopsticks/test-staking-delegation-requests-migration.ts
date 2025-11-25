import "@moonbeam-network/api-augment";
import {
  MoonwallContext,
  beforeAll,
  describeSuite,
  expect,
  type ChopsticksContext,
} from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import type { HexString } from "@polkadot/util/types";
import type { u32 } from "@polkadot/types";
import { hexToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsHex, xxhashAsU8a } from "@polkadot/util-crypto";
import { env } from "node:process";

// When true, the test only reads staking storage and enforces invariants once
// the migration has fully completed (cursor == None). This is useful locally
// to reduce RPC requests and execution time. CI should leave this disabled to run the
// full per-block checks.
const LIGHT_MIGRATION_CHECKS = env.CI !== "true";

const hash = (prefix: HexString, suffix: Uint8Array) => {
  return u8aToHex(u8aConcat(hexToU8a(prefix), xxhashAsU8a(suffix, 64), suffix));
};

const upgradeRestrictionSignal = (paraId: u32) => {
  const prefix = "0xcd710b30bd2eab0352ddcc26417aa194f27bbb460270642b5bcaf032ea04d56a";

  return hash(prefix, paraId.toU8a());
};

const upgradeRuntime = async (context: ChopsticksContext) => {
  const path = (await MoonwallContext.getContext()).rtUpgradePath;
  if (!path) {
    throw new Error("Runtime wasm path (rtUpgradePath) is not configured");
  }

  const rtWasm = await import("node:fs").then(({ readFileSync }) => readFileSync(path));
  const rtHex = `0x${rtWasm.toString("hex")}`;
  const rtHash = blake2AsHex(rtHex);
  const api = context.polkadotJs();
  const signer = context.keyring.alice;

  await context.setStorage({
    module: "system",
    method: "authorizedUpgrade",
    methodParams: `${rtHash}01`, // 01 is for the RT ver check = true
  });
  await context.createBlock();

  await api.tx.system.applyAuthorizedUpgrade(rtHex).signAndSend(signer);

  const paraId: u32 = (await api.query.parachainInfo.parachainId()) as unknown as u32;

  await api.rpc("dev_newBlock", {
    count: 3,
    relayChainStateOverrides: [[upgradeRestrictionSignal(paraId), null]],
  });
};

describeSuite({
  id: "C03",
  timeout: 600_000,
  title: "Chopsticks Staking Migration - DelegationScheduledRequests",
  foundationMethods: "chopsticks",
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();
      await api.isReady;

      const specName = (api.consts.system.version as any).specName.toString();
      log(`Connected to chain specName=${specName}`);
    });

    it({
      id: "T1",
      timeout: 600_000,
      title:
        "DelegationScheduledRequests is migrated from single map to double map and counters initialized",
      test: async () => {
        const psQueryBefore: any = api.query.parachainStaking;

        // 1. Capture the pre-upgrade DelegationScheduledRequests layout (single map).
        const oldEntries = await psQueryBefore.delegationScheduledRequests.entries();
        let totalOldRequests = 0;
        for (const [, boundedVec] of oldEntries as any) {
          const requestsJson = (boundedVec as any).toJSON() as any[];
          totalOldRequests += requestsJson.length;
        }

        log(`Pre-upgrade DelegationScheduledRequests entries (requests): ${totalOldRequests}`);

        const rtBefore = (api.consts.system.version as any).specVersion.toNumber();
        log(`Spec version before upgrade: ${rtBefore}`);

        // 2. Perform the runtime upgrade which includes the new multi-block migration.
        await upgradeRuntime(context as unknown as ChopsticksContext);

        const rtAfter = (api.consts.system.version as any).specVersion.toNumber();
        log(`Spec version after upgrade: ${rtAfter}`);
        expect(rtAfter).to.be.greaterThan(rtBefore);

        // Wait briefly so the API can refresh metadata after the upgrade.
        await new Promise((resolve) => setTimeout(resolve, 1_000));
        await api.isReady;

        // Helper to read the current staking migration state in a single block.
        const readState = async () => {
          const psQueryAfter: any = api.query.parachainStaking;

          const entries = await psQueryAfter.delegationScheduledRequests.entries();
          let totalRequests = 0;
          for (const [, boundedVec] of entries as any) {
            const requestsJson = (boundedVec as any).toJSON() as any[];
            totalRequests += requestsJson.length;
          }

          const perCollatorCounterQuery: any = psQueryAfter.delegationScheduledRequestsPerCollator;
          let totalDelegatorQueues = 0;
          let rawCounters = "n/a";
          if (perCollatorCounterQuery && perCollatorCounterQuery.entries) {
            const counterEntries = await perCollatorCounterQuery.entries();
            totalDelegatorQueues = counterEntries.reduce(
              (acc: number, [, count]: any) => acc + (count as any).toNumber(),
              0
            );
            rawCounters = counterEntries
              .map(
                ([key, count]: any) =>
                  `${key.toString?.() ?? JSON.stringify(key)}=${count.toString()}`
              )
              .join(", ");
          }

          return {
            totalRequests,
            queueCount: entries.length,
            totalDelegatorQueues,
            rawCounters,
          };
        };

        // 3. Progress blocks while the multi-block migrations are running and
        //    assert consistency. In full mode we check invariants on every
        //    block; in light mode we only read staking storage and assert
        //    invariants once the migration has completed.
        const migrationsQuery: any = (api.query as any).migrations;
        let blocksAfterUpgrade = 0;

        // Always check at least one block after the upgrade, then keep going
        // until `pallet-migrations` cursor becomes `None` or we hit a hard cap.
        for (let i = 0; i < 32; i++) {
          await context.createBlock();
          blocksAfterUpgrade += 1;

          const cursor = migrationsQuery?.cursor
            ? await migrationsQuery.cursor()
            : null;

          if (LIGHT_MIGRATION_CHECKS && !(cursor && cursor.isNone)) {
            // Light mode: only log basic progress and wait until the migration
            // reports completion before touching staking storage.
            log(
              `Block +${blocksAfterUpgrade}: LIGHT mode, cursor=${cursor?.toString?.() ?? "n/a"}`
            );
            continue;
          }

          const { totalRequests, queueCount, totalDelegatorQueues, rawCounters } =
            await readState();

          log(
            `Block +${blocksAfterUpgrade}: totalRequests=${totalRequests}, queues=${queueCount}, sumCounters=${totalDelegatorQueues}, rawCounters=${rawCounters}, cursor=${cursor?.toString?.() ?? "n/a"}`
          );

          // In every block (full mode) or at least once at the end (light
          // mode), the migration must preserve the total number of scheduled
          // requests.
          expect(totalRequests).to.equal(
            totalOldRequests,
            "Total number of scheduled delegation requests must be preserved during migration"
          );

          // Once the migrations are finished (`cursor` is None), we expect the
          // per-collator counters to exactly match the number of queues.
          if (cursor && cursor.isNone && queueCount > 0) {
            expect(totalDelegatorQueues).to.equal(
              queueCount,
              "Sum of DelegationScheduledRequestsPerCollator values should equal number of (collator, delegator) queues after migration completes"
            );
            break;
          }
        }
      },
    });
  },
});
