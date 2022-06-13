import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;
const pageSize = (process.env.PAGE_SIZE && parseInt(process.env.PAGE_SIZE)) || 1000;

describeSmokeSuite("Polkadot API - Storage items", { wssUrl, relayWssUrl }, (context) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;

  before("Setup api", async function () {
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
  });

  // This test simply load all the storage items to make sure they can be loaded.
  // It prevents issue where a storage item type is modified but the data is not correctly
  // migrated.
  it("should be decodable", async function () {
    this.timeout(600000); // 1 hour should be enough
    const modules = Object.keys(context.polkadotApi.query);
    for (const moduleName of modules) {
      console.log(`  - ${moduleName}`);
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
          console.log(
            `     - ${fn}: ${count != 0 ? `${chalk.green(`✔`)} [${count} entries]` : "N/A"} `
          );
        } else {
          await module[fn]();
          console.log(`     - ${fn}:  ${chalk.green(`✔`)}`);
        }
      }
    }
  });
});
