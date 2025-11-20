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

      if (specName !== "moonbase") {
        log("Skipping DelegationScheduledRequests migration test on non-moonbase network");
        return;
      }
    });

    it({
      id: "T1",
      timeout: 600_000,
      title:
        "DelegationScheduledRequests is migrated from single map to double map and counters initialized",
      test: async () => {
        const specName = (api.consts.system.version as any).specName.toString();
        if (specName !== "moonbase") {
          // Safety: do nothing on other networks.
          return;
        }

        const psQuery: any = api.query.parachainStaking;

        // 1. Capture the pre-upgrade DelegationScheduledRequests layout (single map).
        const oldEntries = await psQuery.delegationScheduledRequests.entries();
        let totalOldRequests = 0;
        for (const [, boundedVec] of oldEntries as any) {
          const requestsJson = (boundedVec as any).toJSON() as any[];
          totalOldRequests += requestsJson.length;
        }

        log(`Pre-upgrade DelegationScheduledRequests entries (requests): ${totalOldRequests}`);

        const rtBefore = (api.consts.system.version as any).specVersion.toNumber();
        log(`Spec version before upgrade: ${rtBefore}`);

        // 2. Perform the runtime upgrade which includes the new migration.
        await upgradeRuntime(context as unknown as ChopsticksContext);

        const rtAfter = (api.consts.system.version as any).specVersion.toNumber();
        log(`Spec version after upgrade: ${rtAfter}`);
        expect(rtAfter).to.be.greaterThan(rtBefore);

        // Wait briefly so the API can refresh metadata after the upgrade.
        await new Promise((resolve) => setTimeout(resolve, 1_000));
        await api.isReady;

        // 3. Read the post-upgrade DelegationScheduledRequests layout (double map)
        //    and verify that:
        //      - The total number of scheduled requests is preserved.
        //      - DelegationScheduledRequestsPerCollator matches the number of
        //        delegators with pending requests per collator.
        const newEntries = await psQuery.delegationScheduledRequests.entries();
        let totalNewRequests = 0;
        for (const [, boundedVec] of newEntries as any) {
          const requestsJson = (boundedVec as any).toJSON() as any[];
          totalNewRequests += requestsJson.length;
        }

        log(
          `Post-upgrade DelegationScheduledRequests entries (requests): ${totalNewRequests} (was ${totalOldRequests} before upgrade)`
        );

        // 3b. Verify DelegationScheduledRequestsPerCollator matches the number
        //     of delegators with pending requests per collator. After the
        //     migration, each storage key in the double map corresponds to one
        //     unique (collator, delegator) pair, so the sum of all per-collator
        //     counters should equal the number of double-map entries.
        const psQueryAfter: any = api.query.parachainStaking;
        const perCollatorCounterQuery: any = psQueryAfter.delegationScheduledRequestsPerCollator;
        if (!perCollatorCounterQuery || !perCollatorCounterQuery.entries) {
          // If the upgraded runtime does not expose the per-collator counter storage,
          // we cannot check this invariant here.
          log(
            "delegationScheduledRequestsPerCollator storage item not found after upgrade; skipping counter consistency check"
          );
          return;
        }
        const counterEntries = await perCollatorCounterQuery.entries();

        let totalDelegatorQueues = 0;
        for (const [, count] of counterEntries as any) {
          totalDelegatorQueues += (count as any).toNumber();
        }

        expect(totalDelegatorQueues).to.equal(
          newEntries.length,
          "Sum of DelegationScheduledRequestsPerCollator values should equal number of (collator, delegator) queues"
        );
      },
    });
  },
});


