import { Keyring } from "@polkadot/api";
import { expect } from "chai";
import fs from "fs";
import chalk from "chalk";

import { ALITH_PRIV_KEY, ALITH } from "../../util/constants";
import { getRuntimeWasm } from "../../util/para-node";
import { describeParachain } from "../../util/setup-para-tests";

// This test will run on local until the new runtime is available

const runtimeVersion = "local"; // TODO: replace by `runtime-1200`
describeParachain(
  `Runtime upgrade ${runtimeVersion}`,
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
        await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()
      ).unwrap();
      expect(currentVersion.toJSON()).to.deep.equal({
        specVersion: 1103,
        specName: "moonbase",
      });
      console.log(
        `✅ runtime: ${currentVersion.specName.toString()} ${currentVersion.specVersion.toString()}`
      );

      const code = fs.readFileSync(await getRuntimeWasm("moonbase", runtimeVersion)).toString();

      process.stdout.write(
        `Sending sudo.setCode (${code.slice(0, 6)}...${code.slice(-6)} [~${Math.floor(
          code.length / 1024
        )} kb])...`
      );
      const result = await context.polkadotApiParaone.tx.sudo
        .sudoUncheckedWeight(
          await context.polkadotApiParaone.tx.system.setCode(
            fs.readFileSync(await getRuntimeWasm("moonbase", "local")).toString()
          ),
          1
        )
        .signAndSend(alith);
      process.stdout.write(`✅\n`);
      await context.waitBlocks(1);

      const records = await (
        await context.polkadotApiParaone.at(
          await context.polkadotApiParaone.rpc.chain.getBlockHash(context.blockNumber)
        )
      ).query.system.events();
      process.stdout.write("Checking parachainSystem.ValidationFunctionStored...");
      expect(
        records.filter(
          ({ event }) =>
            event.section == "parachainSystem" && event.method == "ValidationFunctionStored"
        )
      ).to.be.length(1);
      process.stdout.write("✅\n");

      process.stdout.write(`Waiting to apply new runtime (${chalk.red(`~4min`)})...`);
      await new Promise<void>(async (resolve) => {
        let isInitialVersion = true;
        const unsub = await context.polkadotApiParaone.rpc.state.subscribeRuntimeVersion(
          async (version) => {
            if (!isInitialVersion) {
              console.log(
                `✅ New runtime: ${version.implName.toString()} ${version.specVersion.toString()}`
              );
              unsub();
              await context.waitBlocks(1); // Wait for next block to have the new runtime applied
              resolve();
            }
            isInitialVersion = false;
          }
        );
      });

      // Uses new API to support new types
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
