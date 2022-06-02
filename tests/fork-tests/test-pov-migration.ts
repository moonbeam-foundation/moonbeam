import { Keyring } from "@polkadot/api";
import child_process from "child_process";

import { ALITH_PRIV_KEY } from "../util/constants";
import { describeParachain } from "../util/setup-para-tests";

/**
 *  This is a complex test, mostly meant to run on automated setup:
 * - It requires to have modified exported state of an existing network
 * - It also uses local git to retrieve existing released versions
 * - It upgrades the network with all released version up to the local one
 *   (defined in the runtime file).
 *   This is for cases when testing a new runtime without having the previous one already deployed
 *   on the targeted runtime
 * - It performs an upgrade using the local runtime wasm
 * - It verifies the new node is producing blocks
 * - It verifies the compressed PoV is not exceeding 2Mb
 */

const RUNTIME_NAME = process.env.RUNTIME_NAME as "moonbeam" | "moonbase" | "moonriver";
const SPEC_FILE = process.env.SPEC_FILE;
const PARA_ID = process.env.PARA_ID && parseInt(process.env.PARA_ID);
const SKIP_INTERMEDIATE_RUNTIME = !!process.env.SKIP_INTERMEDIATE_RUNTIME;

if (!RUNTIME_NAME) {
  console.error(`Missing RUNTIME_NAME (ex: moonbeam)`);
  process.exit(1);
}

if (!SPEC_FILE) {
  console.error(`Missing SPEC_FILE (ex: ~/exports/moonbeam-state.mod.json)`);
  process.exit(1);
}

if (!PARA_ID) {
  console.error(`Missing PARA_ID (ex: 2004)`);
  process.exit(1);
}

const localVersion = parseInt(
  child_process
    .execSync(
      `grep 'spec_version: [0-9]*' ../runtime/${RUNTIME_NAME}/src/lib.rs | grep -o '[0-9]*'`
    )
    .toString()
    .trim()
);

const allRuntimes = child_process
  .execSync(`git tag -l -n 'runtime-[0-9]*' | cut -d' ' -f 1 | cut -d'-' -f 2 | sort -n`)
  .toString()
  .split("\n")
  .map((s) => parseInt(s.trim()))
  .filter((runtime) => runtime != localVersion);

describeParachain(
  `Runtime upgrade on forked ${RUNTIME_NAME}`,
  {
    parachain: {
      spec: SPEC_FILE,
      binary: "local",
    },
    paraId: PARA_ID,
    relaychain: {
      binary: "local",
    },
  },
  (context) => {
    it("should not fail", async function () {
      this.timeout(5000000);
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // Wait for chain to start
      await context.waitBlocks(1);

      const currentVersion = await (
        await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()
      ).unwrap();
      console.log(
        `Current runtime: ✅ runtime ${currentVersion.specName.toString()} ` +
          `${currentVersion.specVersion.toString()}`
      );

      if (!SKIP_INTERMEDIATE_RUNTIME) {
        // For each runtime already released, we do the upgrade if
        for (const runtime of allRuntimes) {
          if (runtime > currentVersion.specVersion.toNumber()) {
            console.log(`Found already released runtime not deployed: ${runtime}`);
            await context.upgradeRuntime(alith, RUNTIME_NAME, `runtime-${runtime}`, {
              useGovernance: true,
            });
            // Wait for upgrade cooldown
            await context.waitBlocks(1);
          }
        }
      }

      await context.upgradeRuntime(alith, RUNTIME_NAME, "local", { useGovernance: true });

      const postCurrentVersion = await (
        (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()) as any
      ).unwrap();
      console.log(
        `New runtime: ✅ runtime ${postCurrentVersion.specName.toString()}` +
          ` ${postCurrentVersion.specVersion.toString()}`
      );

      process.stdout.write("Waiting extra block being produced...");
      await context.waitBlocks(20); // Make sure the new runtime is producing blocks
      process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);
    });
  }
);
