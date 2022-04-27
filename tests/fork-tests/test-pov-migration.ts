import { Keyring } from "@polkadot/api";

import { ALITH_PRIV_KEY } from "../util/constants";
import { describeParachain } from "../util/setup-para-tests";

// This test will run on local until the new runtime is available

const RUNTIME_NAME = process.env.RUNTIME_NAME as "moonbeam" | "moonbase" | "moonriver";
const SPEC_FILE = process.env.SPEC_FILE;
const PARA_ID = process.env.PARA_ID && parseInt(process.env.PARA_ID);

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
      this.timeout(1000000);
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      const currentVersion = await (
        (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()) as any
      ).unwrap();
      console.log(
        `Current runtime: ✅ runtime ${currentVersion.specName.toString()} ${currentVersion.specVersion.toString()}`
      );

      // await context.waitBlocks(400); // Make sure the new runtime is producing blocks
      await context.upgradeRuntime(alith, RUNTIME_NAME, "local", { useGovernance: true });

      const postCurrentVersion = await (
        (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()) as any
      ).unwrap();
      console.log(
        `New runtime: ✅ runtime ${postCurrentVersion.specName.toString()}` +
          ` ${postCurrentVersion.specVersion.toString()}`
      );

      process.stdout.write("Waiting extra block being produced...");
      await context.waitBlocks(4); // Make sure the new runtime is producing blocks
      process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);
    });
  }
);
