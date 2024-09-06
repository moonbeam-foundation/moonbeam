import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { describeSuite, beforeAll } from "@moonwall/cli";
import { ONE_HOURS } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { fail } from "assert";

// Change the following line to reproduce a particular case
const STARTING_KEY_OVERRIDE = null;

const pageSize = (process.env.PAGE_SIZE && parseInt(process.env.PAGE_SIZE)) || 500;

const extractStorageKeyComponents = (storageKey: string) => {
  // The full storage key is composed of
  // - The 0x prefix (2 characters)
  // - The module prefix (32 characters)
  // - The method name (32 characters)
  // - The parameters (variable length)
  const regex = /(?<moduleKey>0x[a-f0-9]{32})(?<fnKey>[a-f0-9]{32})(?<paramsKey>[a-f0-9]*)/i;
  const match = regex.exec(storageKey);

  if (!match) {
    throw new Error("Invalid storage key format");
  }

  const { moduleKey, fnKey, paramsKey } = match.groups!;
  return {
    moduleKey,
    fnKey,
    paramsKey,
  };
};

const randomHex = (nBytes) =>
  [...crypto.getRandomValues(new Uint8Array(nBytes))]
    .map((m) => ("0" + m.toString(16)).slice(-2))
    .join("");

// TODO: This test case really spams the logs, we should find a way to make it less verbose
describeSuite({
  id: "S16",
  title: "Polkadot API - Storage items",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise">;
    let specVersion: number = 0;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
      specVersion = apiAt.consts.system.version.specVersion.toNumber();
    });

    // This test simply load all the storage items to make sure they can be loaded.
    // It prevents issue where a storage item type is modified but the data is not correctly
    // migrated.
    it({
      id: "C100",
      title: "should be decodable",
      timeout: ONE_HOURS,
      test: async function () {
        let currentStartKey = "";
        const modules = Object.keys(paraApi.query);
        for (const moduleName of modules) {
          log(`  - ${moduleName}`);
          const module = apiAt.query[moduleName];
          const fns = Object.keys(module);
          for (const fn of fns) {
            log(`🔎 checking ${moduleName}::${fn}`);
            const keys = Object.keys(module[fn]);
            try {
              if (keys.includes("keysPaged")) {
                // Generate a first query with an empty startKey
                currentStartKey = "";
                const emptyKeyEntries = await module[fn].entriesPaged({
                  args: [],
                  pageSize,
                  startKey: currentStartKey,
                });

                // Skip if no entries are found
                if (emptyKeyEntries.length === 0) {
                  log(`     - ${fn}:  ${chalk.green(`✔ No entries found`)}`);
                  continue;
                }
                // Skip if all entries are checked
                if (emptyKeyEntries.length < pageSize) {
                  log(
                    `     - ${fn}:  ${chalk.green(
                      `✔ All ${emptyKeyEntries.length} entries checked`
                    )}`
                  );
                  continue;
                }
                // Log emptyKeyFirstEntry
                const emptyKeyFirstEntryKey = emptyKeyEntries[0][0].toString();
                log(`     - ${fn}:  ${chalk.green(`🔎`)} (first key : ${emptyKeyFirstEntryKey})`);

                // If there are more entries, perform a random check
                // 1. Get the first entry storage key
                const firstEntry = emptyKeyEntries[0];
                const storageKey = firstEntry[0].toString();

                // 2. Extract the module, fn and params keys
                const { moduleKey, fnKey, paramsKey } = extractStorageKeyComponents(storageKey);

                // 3. Generate a random startKey, will be overridden if STARTING_KEY_OVERRIDE is set
                currentStartKey = moduleKey + fnKey + randomHex(paramsKey.length);
                currentStartKey = STARTING_KEY_OVERRIDE || currentStartKey;

                // 4. Fetch the storage entries with the random startKey
                // Trying to decode all storage entries may cause the node to timeout, decoding
                // random storage entries should be enough to verify if a storage migration
                // was missed.
                const randomEntries = await module[fn].entriesPaged({
                  args: [],
                  pageSize,
                  startKey: currentStartKey,
                });
                // Log first entry storage key
                const firstRandomEntryKey = randomEntries[0][0].toString();
                log(`     - ${fn}:  ${chalk.green(`🔎`)} (random key: ${firstRandomEntryKey})`);
              } else if (fn != "code") {
                await module[fn]();
              }

              log(`     - ${fn}:  ${chalk.green(`✔`)}`);
            } catch (e) {
              const failMsg = `Failed to fetch storage at (${moduleName}::${fn}) `;
              const RNGDetails = `using startKey "${currentStartKey} at block ${atBlockNumber}`;
              const msg = chalk.red(`${failMsg} ${RNGDetails}`);
              log(msg, e);
              const reproducing = `To reproduce this failled case, set the STARTING_KEY_OVERRIDE 
              variable to "${currentStartKey}" at the top of the test file and run the test again.`;
              log(chalk.red(reproducing));
              fail(msg);
            }
          }
        }
      },
    });
  },
});
