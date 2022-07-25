import { expect } from "chai";
import child_process from "child_process";
import { alith } from "../../util/accounts";

import { describeParachain } from "../../util/setup-para-tests";

// This test will run on local until the new runtime is available

const localVersion = child_process
  .execSync(`grep 'spec_version: [0-9]*' ../runtime/moonbase/src/lib.rs | grep -o '[0-9]*'`)
  .toString()
  .trim();

const localAuthoringVersion = child_process
  .execSync(`grep 'authoring_version: [0-9]*' ../runtime/moonbase/src/lib.rs | grep -o '[0-9]*'`)
  .toString()
  .trim();

let alreadyReleased = "";
try {
  alreadyReleased = child_process
    .execSync(
      `git tag -l -n 'runtime-[0-9]*' | cut -d' ' -f 1 | cut -d'-' -f 2 | grep "${localVersion}"`
    )
    .toString()
    .trim();
} catch (e) {
  alreadyReleased = "";
}

let baseRuntime: string;
// Determines if the test can run previous runtime with the actual client
// if true the test will not run.
let hasAuthoringChanges: boolean = false;
if (localVersion == alreadyReleased) {
  console.log(`${localVersion} already released. Skipping current runtime `);
  baseRuntime = localVersion;
} else {
  // Retrieves previous version
  baseRuntime = child_process
    .execSync(
      `git tag -l -n 'runtime-[0-9]*' | cut -d' ' -f 1 | cut -d'-' -f 2 ` +
        `| sed '1 i ${localVersion}' | sort -n -r ` +
        `| uniq | grep -A1 "${localVersion}" | tail -1`
    )
    .toString()
    .trim();

  hasAuthoringChanges =
    child_process
      .execSync(
        `git grep authoring_version ` +
          `$(git rev-list runtime-1606..HEAD -- ../runtime/moonbase/src/lib.rs) ` +
          `-- ../runtime/moonbase/src/lib.rs ` +
          `| grep -v "$(git grep authoring_version runtime-1606 ` +
          `-- ../runtime/moonbase/src/lib.rs ` +
          `| grep -o 'authoring_version:\ *[0-9]')" ` +
          `| grep -o 'authoring_version:\ *[0-9]*'`
      )
      .toString()
      .trim().length > 0;
}

console.log(`Using base runtime ${baseRuntime} (authoring changes: ${hasAuthoringChanges})`);

const RUNTIME_VERSION = "local";
describeParachain(
  `Runtime upgrade ${RUNTIME_VERSION}`,
  {
    parachain: {
      chain: "moonbase-local",
      runtime: `runtime-${baseRuntime}`,
      binary: "local",
    },
    relaychain: {
      binary: "local",
    },
  },
  (context) => {
    if (localVersion !== alreadyReleased && !hasAuthoringChanges) {
      it("should not fail", async function () {
        // Expected to take 10 blocks for upgrade + 4 blocks to check =>
        // ~200000 + init 60000 + error marging 140000
        this.timeout(400000);

        const currentVersion = await (
          (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()) as any
        ).unwrap();
        expect(currentVersion.toJSON()).to.deep.equal({
          specVersion: Number(baseRuntime),
          specName: "moonbase",
        });
        console.log(
          `Current runtime: ✅ runtime ${currentVersion.specName.toString()} ` +
            `${currentVersion.specVersion.toString()}`
        );

        await context.upgradeRuntime(alith, "moonbase", RUNTIME_VERSION);

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
