import { expect } from "chai";
import { describeParachain, retrieveParaVersions } from "../../util/setup-para-tests";

// This test will run on local until the new runtime is available
const RUNTIME_VERSION = "local";
const { localVersion, previousVersion, hasAuthoringChanges } = retrieveParaVersions();
describeParachain(
  `Runtime upgrade ${RUNTIME_VERSION}`,
  {
    parachain: {
      chain: "moonbase-local",
      runtime: `runtime-${previousVersion}`,
      binary: "local",
    },
    relaychain: {
      binary: "local",
    },
  },
  (context) => {
    if (localVersion !== previousVersion && !hasAuthoringChanges) {
      it("should not fail", async function () {
        // Expected to take 10 blocks for upgrade + 4 blocks to check =>
        // ~200000 + init 60000 + error marging 140000
        this.timeout(400000);

        const currentVersion = await (
          (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()) as any
        ).unwrap();
        expect(currentVersion.toJSON()).to.deep.equal({
          specVersion: Number(previousVersion),
          specName: "moonbase",
        });
        console.log(
          `Current runtime: ✅ runtime ${currentVersion.specName.toString()} ` +
            `${currentVersion.specVersion.toString()}`
        );

        await context.upgradeRuntime({ runtimeName: "moonbase", runtimeTag: RUNTIME_VERSION });

        process.stdout.write(`Checking on-chain runtime version ${localVersion}...`);
        expect(
          await (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()).toJSON()
        ).to.deep.equal({
          specVersion: Number(localVersion),
          specName: "moonbase",
        });
        process.stdout.write("✅\n");

        process.stdout.write("Waiting extra block being produced...");
        await context.waitBlocks(4); // Make sure the new runtime is producing blocks
        process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);
      });
    }
  }
);
