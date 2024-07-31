import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { describeSuite, beforeAll } from "@moonwall/cli";
import { ONE_HOURS } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { fail } from "assert";

const pageSize = (process.env.PAGE_SIZE && parseInt(process.env.PAGE_SIZE)) || 500;

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
                const startKey = "";
                // Trying to decode all storage entries may cause the node to timeout, decoding
                // the first storage entries should be enough to verify if a storage migration
                // was missed.
                await module[fn].entriesPaged({
                  args: [],
                  pageSize,
                  startKey,
                });
              } else if (fn != "code") {
                await module[fn]();
              }

              log(`     - ${fn}:  ${chalk.green(`✔`)}`);
            } catch (e) {
              const msg = chalk.red(`Failed to fetch storage at (${moduleName}::${fn})`);
              log(msg, e);
              fail(msg);
            }
          }
        }
      },
    });
  },
});
