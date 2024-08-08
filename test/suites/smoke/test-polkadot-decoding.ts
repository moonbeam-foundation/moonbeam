import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { describeSuite, beforeAll } from "@moonwall/cli";
import { ONE_HOURS } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { fail } from "assert";

const pageSize = (process.env.PAGE_SIZE && parseInt(process.env.PAGE_SIZE)) || 500;

const extractStorageKeyComponents = (storageKey: string) => {
  // The full storage key is composed of
  // - The 0x prefix (2 characters)
  // - The module prefix (32 characters)
  // - The method name (32 characters)
  // - The parameters (variable length)
  const moduleKey = storageKey.substring(0, 2 + 32); // Includes the 0x prefix
  const fnKey = storageKey.substring(2 + 32, 2 + 32 + 32);
  const paramsKey = storageKey.substring(2 + 32 + 32);
  return {
    moduleKey,
    fnKey,
    paramsKey,
  };
};

// A PRNG used for generating random numbers in a deterministic way
class SeededPRNG {
  private seed: number;
  private modulus: number;
  private multiplier: number;
  private increment: number;

  constructor(seed: number) {
    this.seed = seed;
    this.modulus = 2 ** 31 - 1; // A large prime number
    this.multiplier = 48271; // A commonly used multiplier
    this.increment = 0; // Increment is often set to 0 in LCG
  }

  // Get the current seed
  public getSeed(): number {
    return this.seed;
  }

  // Generate the next random number
  private next(): number {
    this.seed = (this.multiplier * this.seed + this.increment) % this.modulus;
    return this.seed;
  }

  // Get a random number within a range [min, max)
  public randomInRange(min: number, max: number): number {
    const randomValue = this.next() / this.modulus;
    return Math.floor(randomValue * (max - min) + min);
  }

  // Get a random hex string of a given length
  public randomHex(length: number): string {
    const hexChars = "0123456789abcdef";
    let result = "";
    for (let i = 0; i < length; i++) {
      result += hexChars[this.randomInRange(0, hexChars.length)];
    }
    return result;
  }
}

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
    let PRNG = new SeededPRNG(42);

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
      specVersion = apiAt.consts.system.version.specVersion.toNumber();
      // Initialize PRNG with current timestamp
      PRNG = new SeededPRNG(new Date().getTime());
      log(`Initializing PRNG with seed "${PRNG.getSeed()}"`);
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
        let currentSeed = PRNG.getSeed();
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

                // 3. Generate a random startKey
                // Overwrite the PRNG seed to check particular cases by
                // uncommenting the following line
                // PRNG = new SeededPRNG(42);
                currentSeed = PRNG.getSeed();
                currentStartKey = moduleKey + fnKey + PRNG.randomHex(paramsKey.length);

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
              const PRNGDetails = `using seed "${currentSeed}" and startKey "${currentStartKey}"`;
              const msg = chalk.red(`${failMsg} ${PRNGDetails}`);
              log(msg, e);
              fail(msg);
            }
          }
        }
      },
    });
  },
});
