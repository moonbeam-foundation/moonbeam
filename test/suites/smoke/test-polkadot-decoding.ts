import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ONE_HOURS } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
const pageSize = (process.env.PAGE_SIZE && parseInt(process.env.PAGE_SIZE)) || 500;

// TODO: This test case really spams the logs, we should find a way to make it less verbose
describeSuite({
  id: "S1500",
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
          if (moduleName == "system") {
            // We skip system because too big and tested in other places too
            continue;
          }
          for (const fn of fns) {
            if (moduleName == "evm" && ["accountStorages", "accountCodes"].includes(fn)) {
              // This is just H256 entries and quite big
              continue;
            }
            if (
              moduleName == "parachainStaking" &&
              ["atStake"].includes(fn) &&
              specVersion == 1901
            ) {
              // AtStake is broken in 1902 until a script is run
              continue;
            }

            const keys = Object.keys(module[fn]);
            if (keys.includes("keysPaged")) {
              // Map item
              let startKey = "";
              let count = 0;
              while (true) {
                let query = await module[fn].entriesPaged({
                  args: [],
                  pageSize,
                  startKey,
                });

                if (query.length == 0) {
                  break;
                }
                count += query.length;
                startKey = query[query.length - 1][0].toString();
              }
              log(
                `     - ${fn}: ${count != 0 ? `${chalk.green(`✔`)} [${count} entries]` : "N/A"} `
              );
            } else {
              await module[fn]();
              log(`     - ${fn}:  ${chalk.green(`✔`)}`);
            }
          }
        }
      },
    });
  },
});
