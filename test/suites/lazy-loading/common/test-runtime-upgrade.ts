import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { RUNTIME_CONSTANTS } from "../../../helpers";
import type { ApiPromise } from "@polkadot/api";
import fs from "node:fs/promises";
import { u8aToHex } from "@polkadot/util";
import assert from "node:assert";
import type { SpRuntimeDispatchError } from "@polkadot/types/lookup";

describeSuite({
  id: "LL-COMMON-RT-UPGRADE",
  title: "Lazy Loading - Runtime Upgrade",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();

      const runtimeChain = api.runtimeChain.toUpperCase();
      const runtime = runtimeChain
        .split(" ")
        .filter((v) => Object.keys(RUNTIME_CONSTANTS).includes(v))
        .join()
        .toLowerCase();
      const wasmPath = `../target/release/wbuild/${runtime}-runtime/${runtime}_runtime.compact.compressed.wasm`;
      const runtimeWasmHex = u8aToHex(await fs.readFile(wasmPath));

      const rtBefore = api.consts.system.version.specVersion.toNumber();
      log(`Current runtime: ${rtBefore}`);
      log(`About to upgrade to runtime at: ${wasmPath}`);

      await context.createBlock();
      const { result } = await context.createBlock(
        api.tx.system.applyAuthorizedUpgrade(runtimeWasmHex)
      );

      assert(result, "Block has no extrinsic results");
      const errors = result.events
        // find/filter for failed events
        .filter(({ event }) => api.events.system.ExtrinsicFailed.is(event))
        // we know that data for system.ExtrinsicFailed is
        // (DispatchError, DispatchInfo)
        .map(
          ({
            event: {
              data: [error],
            },
          }) => {
            const dispatchError = error as SpRuntimeDispatchError;
            if (dispatchError.isModule) {
              // for module errors, we have the section indexed, lookup
              const decoded = api.registry.findMetaError(dispatchError.asModule);
              const { docs, method, section } = decoded;

              return `${section}.${method}: ${docs.join(" ")}`;
            }
            // Other, CannotLookup, BadOrigin, no extra info
            return error.toString();
          }
        );

      if (errors.length) {
        throw new Error(`Could not upgrade runtime. \nErrors:\n\n\t- ${errors.join("\n\t-")}\n`);
      }

      // This next block will receive the GoAhead signal
      await context.createBlock();
      // The next block will process the runtime upgrade
      await context.createBlock();

      const rtAfter = api.consts.system.version.specVersion.toNumber();
      log(`RT upgrade has increased specVersion from ${rtBefore} to ${rtAfter}`);

      expect(rtBefore).to.be.not.equal(rtAfter, "Runtime upgrade failed");

      const specName = api.consts.system.version.specName.toString();
      log(`Currently connected to chain: ${specName}`);
    });

    it({
      id: "T01",
      timeout: 600_000, // 10 minutes
      title: "Ensure migrations are executed",
      test: async function () {
        // Ensure multi block migrations started
        const upgradeStartedEvt = (await api.query.system.events()).find(({ event }) =>
          api.events.multiBlockMigrations.UpgradeStarted.is(event)
        );

        // Ensure single block migrations were executed
        const versionMigrationFinishedEvt = (await api.query.system.events()).find(({ event }) =>
          api.events.polkadotXcm.VersionMigrationFinished.is(event)
        );
        expect(!!versionMigrationFinishedEvt, "Permanent XCM migration was executed").to.be.true;

        // If there are any multi-block migrations, confirm that they are advancing
        if (upgradeStartedEvt) {
          const migrationAdvancedEvt = (await api.query.system.events()).find(({ event }) =>
            api.events.multiBlockMigrations.MigrationAdvanced.is(event)
          );
          expect(!!migrationAdvancedEvt, "Migration Advanced").to.be.true;

          // Ensure multi block migrations completed in less than 20 blocks.
          // We don't assume a fixed number of MigrationCompleted events:
          // there may be one per multi-block migration. Instead, we require
          // that at least one MigrationCompleted is observed and that
          // UpgradeCompleted eventually fires, which indicates that all
          // migrations have finished.
          let sawMigrationCompleted = false;
          let sawUpgradeCompleted = false;
          for (let attempts = 0; attempts < 20; attempts++) {
            const events = await api.query.system.events();
            if (
              events.some(({ event }) =>
                api.events.multiBlockMigrations.MigrationCompleted.is(event)
              )
            ) {
              sawMigrationCompleted = true;
            }
            if (
              events.some(({ event }) => api.events.multiBlockMigrations.UpgradeCompleted.is(event))
            ) {
              sawUpgradeCompleted = true;
              break;
            }
            await context.createBlock();
          }

          expect(sawMigrationCompleted, "At least one MigrationCompleted event should be observed")
            .to.be.true;
          expect(sawUpgradeCompleted, "UpgradeCompleted event should be observed").to.be.true;
        }
      },
    });
  },
});
