import { expect } from "chai";
import Web3 from "web3";
import { baltathar, alith, BALTATHAR_PRIVATE_KEY } from "../../util/accounts";
import { describeParachain, retrieveParaVersions } from "../../util/setup-para-tests";
import { customWeb3Request } from "../../util/providers";
const debug = require("debug")("para:migration-txpool");

const RUNTIME_VERSION = "local";
const { localVersion, previousVersion, hasAuthoringChanges } = retrieveParaVersions();
describeParachain(
  `TxPool during Runtime Migration ${RUNTIME_VERSION}`,
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
      it("should skip transactions", async function () {
        this.timeout(400000);

        // This process will generate transfer at regular interval to check
        // the runtime migration block doesn't contain txs
        const stopNewTxs = (() => {
          let stopLoop = false;
          let promises = [];
          (async () => {
            let baltatharNonce = await context.web3.eth.getTransactionCount(baltathar.address);
            // Send a transfer every 6s to fill the txpool
            while (!stopLoop) {
              const tx = await context.web3.eth.accounts.signTransaction(
                {
                  from: baltathar.address,
                  to: alith.address,
                  value: Web3.utils.toWei("1", "ether"),
                  gasPrice: Web3.utils.toWei("10", "Gwei"),
                  gas: "0x100000",
                  nonce: baltatharNonce++,
                },
                BALTATHAR_PRIVATE_KEY
              );

              await new Promise((resolve) => setTimeout(resolve, 3000));
              promises.push(
                customWeb3Request(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]).then(
                  (e) => {
                    if (e.error) {
                      console.log(e.error);
                    }
                  }
                )
              );
            }
          })();
          return async () => {
            stopLoop = true;
            // Returns when all the txs have been processed
            process.stdout.write(`Waiting for all txs to finish: ${promises.length}...`);
            await Promise.all(promises);
            process.stdout.write("✅\n");
            return;
          };
        })();

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

        // Upgrade and retrieved block number of the runtime being applied
        const upgradeBlockNumber = await context.upgradeRuntime({
          runtimeName: "moonbase",
          runtimeTag: RUNTIME_VERSION,
        });
        const migrationBlockNumber = upgradeBlockNumber + 1;

        process.stdout.write(`Checking on-chain runtime version ${localVersion}...`);
        expect(
          await (await context.polkadotApiParaone.query.system.lastRuntimeUpgrade()).toJSON()
        ).to.deep.equal({
          specVersion: Number(localVersion),
          specName: "moonbase",
        });
        process.stdout.write(`✅: on block ${upgradeBlockNumber}\n`);

        // Stop the transactions and wait for 1 additional block to check if
        // transactions are still included after the migration
        await stopNewTxs();
        await context.waitBlocks(1);

        process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);

        // Check blocks before/during/after the runtime upgrade and migration
        for (let i = upgradeBlockNumber - 1; i <= migrationBlockNumber + 1; i++) {
          const blockHash = await context.polkadotApiParaone.rpc.chain.getBlockHash(i);
          const [{ block }, records] = await Promise.all([
            context.polkadotApiParaone.rpc.chain.getBlock(blockHash),
            await (await context.polkadotApiParaone.at(blockHash)).query.system.events(),
          ]);

          const extrinsicCountByMethod = block.extrinsics
            .map((e) => e.method.method)
            .reduce((p, v) => {
              p[v] = (p[v] || 0) + 1;
              return p;
            }, {});
          switch (i) {
            case upgradeBlockNumber:
              // This is unknown but observed behavior where runtime applied has block limit high
              // enough to prevent other extrinsic to be included
              expect(records.find((r) => r.event.method == "ValidationFunctionApplied")).to.not.be
                .null;
              expect(block.extrinsics.length).to.be.equal(4);
              expect(extrinsicCountByMethod["transact"]).to.be.undefined;
              break;
            case migrationBlockNumber:
              // Introduced in PR2006 to prevent XCM/User Tx during migration
              expect(records.find((r) => r.event.method == "RuntimeUpgradeStarted")).to.not.be.null;
              expect(records.find((r) => r.event.method == "RuntimeUpgradeCompleted")).to.not.be
                .null;
              expect(block.extrinsics.length).to.be.equal(4);
              expect(extrinsicCountByMethod["transact"]).to.be.undefined;
              break;
            default: // other blocks should contain transactions
              expect(block.extrinsics.length).to.be.above(4);
              expect(extrinsicCountByMethod["transact"]).to.be.above(0);
          }

          debug(
            `#${i.toString().padEnd(3, " ")}: ${block.extrinsics.length
              .toString()
              .padEnd(2, " ")} (${Object.entries(extrinsicCountByMethod)
              .map(
                ([key, count]) =>
                  `${key.slice(0, 8).padStart(8, " ")}: ${count.toString().padStart(2, " ")}`
              )
              .join(" - ")}) ${
              records.find((r) => r.event.method == "ValidationFunctionApplied")
                ? "ValidationFunctionApplied"
                : records.find((r) => r.event.method == "RuntimeUpgradeStarted")
                ? "RuntimeUpgradeStarted"
                : ""
            }`
          );
        }
      });
    }
  }
);
