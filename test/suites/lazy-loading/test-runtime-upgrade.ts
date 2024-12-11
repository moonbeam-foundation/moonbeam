import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { RUNTIME_CONSTANTS } from "../../helpers";
import { ApiPromise } from "@polkadot/api";
import fs from "fs/promises";
import { u8aToHex } from "@polkadot/util";
import assert from "node:assert";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";

describeSuite({
  id: "LD01",
  title: "Lazy Loading - Runtime Upgrade",
  foundationMethods: "dev",
  options: {
    forkConfig: {
      url: process.env.FORK_URL ?? "https://moonbeam.unitedbloc.com",
      stateOverridePath: "tmp/lazyLoadingStateOverrides.json",
      verbose: true,
    },
  },
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();

      const runtimeChain = api.consts.system.version.specName.toUpperCase();
      const runtime = runtimeChain
        .split(" ")
        .filter((v) => Object.keys(RUNTIME_CONSTANTS).includes(v))
        .join()
        .toLowerCase();
      const wasmPath = `../target/release/wbuild/${runtime}-runtime/${runtime}_runtime.compact.compressed.wasm`; // editorconfig-checker-disable-line

      const runtimeWasmHex = u8aToHex(await fs.readFile(wasmPath));

      const rtBefore = api.consts.system.version.specVersion.toNumber();
      log("Current runtime:", rtBefore);
      log("About to upgrade to runtime at:", wasmPath);

      await context.createBlock([], { finalize: false });
      const { result } = await context.createBlock(
        api.tx.system.applyAuthorizedUpgrade(runtimeWasmHex),
        { finalize: false }
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
              data: [error, info],
            },
          }) => {
            const dispatchError = error as SpRuntimeDispatchError;
            if (dispatchError.isModule) {
              // for module errors, we have the section indexed, lookup
              const decoded = api.registry.findMetaError(dispatchError.asModule);
              const { docs, method, section } = decoded;

              return `${section}.${method}: ${docs.join(" ")}`;
            } else {
              // Other, CannotLookup, BadOrigin, no extra info
              return error.toString();
            }
          }
        );

      if (errors.length) {
        throw new Error(`Could not upgrade runtime. \nErrors:\n\n\t- ${errors.join("\n\t-")}\n`);
      }

      // This next block will receive the GoAhead signal
      await context.createBlock([], { finalize: false });
      // The next block will process the runtime upgrade
      await context.createBlock([], { finalize: false });

      const events = (await api.query.system.events()).filter(({ event }) =>
        api.events.migrations.RuntimeUpgradeCompleted.is(event)
      );
      expect(events.length > 0, "Migrations should complete").to.be.true;

      const rtAfter = api.consts.system.version.specVersion.toNumber();
      log(`RT upgrade has increased specVersion from ${rtBefore} to ${rtAfter}`);

      expect(rtBefore).to.be.not.equal(rtAfter, "Runtime upgrade failed");

      const specName = api.consts.system.version.specName.toString();
      log(`Currently connected to chain: ${specName}`);
    });

    it({
      id: "T01",
      title: "Validate new applied runtime",
      test: async function () {
        // TODO
      },
    });
  },
});
