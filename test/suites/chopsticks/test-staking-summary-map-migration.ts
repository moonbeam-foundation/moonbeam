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

// Index of the summary-map migration in the `MultiBlockMigrations` tuple
// defined in `runtime/common/src/migrations.rs`.
const SUMMARY_MAP_MIGRATION_INDEX = 3;

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

// Represents a summary map entry: either a revoke or an aggregated decrease total.
type ExpectedSummary = { revoke: bigint } | { decrease: bigint };

describeSuite({
  id: "C04",
  title: "Chopsticks Staking Migration - DelegationScheduledRequestsSummaryMap",
  foundationMethods: "chopsticks",
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;
    let specName: string;

    beforeAll(async () => {
      api = context.polkadotJs();
      await api.isReady;

      specName = (api.consts.system.version as any).specName.toString();
      log(`Connected to chain specName=${specName}`);
    }, 120_000);

    it({
      id: "T1",
      timeout: 900_000,
      title: "Summary map populated with correct revoke/decrease entries after migration",
      test: async () => {
        const isMoonbeam = specName === "moonbeam";
        const psQueryBefore: any = api.query.parachainStaking;

        // 1. Capture the pre-upgrade DelegationScheduledRequests.
        // The storage is already in double-map format (collator, delegator) -> Vec<{action, ...}>
        // because the double-map migration ran in a prior upgrade.
        const oldEntries = await psQueryBefore.delegationScheduledRequests.entries();
        let totalOldRequests = 0;
        const expectedPerCollator: Record<string, number> = {};
        const expectedSummaries = new Map<string, ExpectedSummary>();

        // Detect format from the first entry's key args count.
        const isDoubleMap =
          oldEntries.length > 0 && ((oldEntries[0] as any)[0] as any).args?.length === 2;

        log(`Pre-upgrade storage format: ${isDoubleMap ? "double-map" : "single-map"}`);

        if (isDoubleMap) {
          // Double-map format: key = (collator, delegator), value = Vec<{whenExecutable, action}>
          const delegatorsPerCollator = new Map<string, Set<string>>();

          for (const [storageKey, boundedVec] of oldEntries as any) {
            const requestsJson = (boundedVec as any).toJSON() as any[];
            totalOldRequests += requestsJson.length;

            const collator = (storageKey as any).args?.[0]?.toString?.() ?? "";
            const delegator = (storageKey as any).args?.[1]?.toString?.() ?? "";
            if (!collator || !delegator) continue;

            if (!delegatorsPerCollator.has(collator)) {
              delegatorsPerCollator.set(collator, new Set());
            }
            delegatorsPerCollator.get(collator)!.add(delegator);

            const key = `${collator}|${delegator}`;
            const revokeReq = requestsJson.find((r: any) => r.action?.revoke != null);
            if (revokeReq) {
              expectedSummaries.set(key, { revoke: BigInt(revokeReq.action.revoke) });
            } else {
              let total = 0n;
              for (const r of requestsJson) {
                if (r.action?.decrease != null) {
                  total += BigInt(r.action.decrease);
                }
              }
              if (total > 0n) {
                expectedSummaries.set(key, { decrease: total });
              }
            }
          }

          for (const [collator, delegators] of delegatorsPerCollator) {
            expectedPerCollator[collator] = delegators.size;
          }
        } else {
          // Single-map format: key = collator, value = Vec<{delegator, whenExecutable, action}>
          for (const [storageKey, boundedVec] of oldEntries as any) {
            const requestsJson = (boundedVec as any).toJSON() as any[];
            totalOldRequests += requestsJson.length;

            const collator = (storageKey as any).args?.[0]?.toString?.() ?? "";
            if (!collator) continue;

            const byDelegator = new Map<string, any[]>();
            const uniqueDelegators = new Set<string>();

            for (const req of requestsJson as any[]) {
              const delegator = (req as any)?.delegator;
              if (delegator == null) continue;
              const delegatorStr = String(delegator);
              uniqueDelegators.add(delegatorStr);
              if (!byDelegator.has(delegatorStr)) {
                byDelegator.set(delegatorStr, []);
              }
              byDelegator.get(delegatorStr)!.push(req);
            }

            expectedPerCollator[collator] = uniqueDelegators.size;

            for (const [delegator, reqs] of byDelegator) {
              const key = `${collator}|${delegator}`;
              const revokeReq = reqs.find((r: any) => r.action?.revoke != null);
              if (revokeReq) {
                expectedSummaries.set(key, { revoke: BigInt(revokeReq.action.revoke) });
              } else {
                let total = 0n;
                for (const r of reqs) {
                  if (r.action?.decrease != null) {
                    total += BigInt(r.action.decrease);
                  }
                }
                if (total > 0n) {
                  expectedSummaries.set(key, { decrease: total });
                }
              }
            }
          }
        }

        log(`Pre-upgrade DelegationScheduledRequests entries (requests): ${totalOldRequests}`);
        log(`Expected DelegationScheduledRequestsSummaryMap entries: ${expectedSummaries.size}`);

        const rtBefore = (api.consts.system.version as any).specVersion.toNumber();
        log(`Spec version before upgrade: ${rtBefore}`);

        // 2. Perform the runtime upgrade.
        await upgradeRuntime(context as unknown as ChopsticksContext);

        const rtAfter = (api.consts.system.version as any).specVersion.toNumber();
        log(`Spec version after upgrade: ${rtAfter}`);
        expect(rtAfter).to.be.greaterThan(rtBefore);

        await new Promise((resolve) => setTimeout(resolve, 1_000));
        await api.isReady;

        // 3. Progress blocks until the summary-map migration completes.
        const migrationsQuery: any = (api.query as any).multiBlockMigrations;
        let blocksAfterUpgrade = 0;
        let sawSummaryMigration = false;

        for (let i = 0; i < 128; i++) {
          await context.createBlock();
          blocksAfterUpgrade += 1;

          const cursor = migrationsQuery?.cursor ? await migrationsQuery.cursor() : null;
          const cursorStr = cursor?.toString?.() ?? "n/a";

          const cursorJson = cursor?.toJSON?.() as any;
          const activeIndex: number | null =
            cursorJson?.active && typeof cursorJson.active.index === "number"
              ? (cursorJson.active.index as number)
              : null;

          if (activeIndex === SUMMARY_MAP_MIGRATION_INDEX) {
            sawSummaryMigration = true;
          }

          const isCursorNone = !!cursor && (cursor as any).isNone;

          if (!isCursorNone) {
            log(`Block +${blocksAfterUpgrade}: cursor=${cursorStr}`);
            continue;
          }

          // All multi-block migrations have finished.
          if (!sawSummaryMigration) {
            throw new Error(
              "Summary map migration did not appear in multiBlockMigrations cursor before completion"
            );
          }

          // 4. Read and verify summary map entries.
          const psQueryAfter: any = api.query.parachainStaking;

          const summaryQuery: any = psQueryAfter.delegationScheduledRequestsSummaryMap;
          const summaryEntries: Map<string, any> = new Map();
          if (summaryQuery?.entries) {
            const entries = await summaryQuery.entries();
            for (const [storageKey, value] of entries as any) {
              const collator = (storageKey as any).args?.[0]?.toString?.() ?? "";
              const delegator = (storageKey as any).args?.[1]?.toString?.() ?? "";
              if (collator && delegator) {
                summaryEntries.set(`${collator}|${delegator}`, (value as any).toJSON());
              }
            }
          }

          log(
            `DelegationScheduledRequestsSummaryMap entries after migration: ${summaryEntries.size}`
          );
          expect(summaryEntries.size).to.equal(
            expectedSummaries.size,
            "Summary map entry count must match expected (one per collator/delegator pair with requests)"
          );

          // On non-Moonbeam networks, verify each summary entry's content.
          if (!isMoonbeam) {
            for (const [key, expected] of expectedSummaries) {
              const actual = summaryEntries.get(key);
              expect(actual).to.not.be.undefined;

              if ("revoke" in expected) {
                expect(actual?.revoke != null).to.be.true;
                expect(BigInt(actual.revoke)).to.equal(
                  expected.revoke,
                  `Summary[${key}] revoke amount mismatch`
                );
              } else {
                expect(actual?.decrease != null).to.be.true;
                expect(BigInt(actual.decrease)).to.equal(
                  expected.decrease,
                  `Summary[${key}] decrease total mismatch`
                );
              }
            }
          }

          log(`Summary map migration verified after ${blocksAfterUpgrade} blocks post-upgrade`);
          break;
        }
      },
    });
  },
});
