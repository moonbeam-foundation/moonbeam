import { Keyring } from "@polkadot/api";
import { expect } from "chai";

import { ALITH_PRIV_KEY } from "../../util/constants";
import { describeParachain } from "../../util/setup-para-tests";

// This test will run on local until the new runtime is available

const RUNTIME_VERSION = "local"; // TODO: replace by `runtime-1200`
describeParachain(
  `Runtime upgrade ${RUNTIME_VERSION}`,
  { chain: "moonbase-local", runtime: "runtime-1103", binary: "local" },
  (context) => {
    it("should not fail", async function () {
      // Expected to take 10 blocks for upgrade + 4 blocks to check =>
      // ~200000 + init 60000 + error marging 140000
      this.timeout(400000);
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      process.stdout.write("Checking current runtime...");
      const currentVersion = await (
        (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()) as any
      ).unwrap();
      expect(currentVersion.toJSON()).to.deep.equal({
        specVersion: 1103,
        specName: "moonbase",
      });
      console.log(
        `✅ runtime ${currentVersion.specName.toString()} ${currentVersion.specVersion.toString()}`
      );

      await context.upgradeRuntime(alith, "moonbase", RUNTIME_VERSION);
      const newApi = await context.createPolkadotApiParachain(0);

      process.stdout.write("Checking on-chain runtime version 1200...");
      expect(await (await newApi.query.system.lastRuntimeUpgrade()).toJSON()).to.deep.equal({
        specVersion: 1200,
        specName: "moonbase",
      });
      process.stdout.write("✅\n");

      process.stdout.write("Waiting extra block being produced...");
      await context.waitBlocks(2); // Make sure the new runtime is producing blocks
      process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);
    });
  }
);
