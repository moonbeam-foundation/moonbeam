import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { THIRTY_MINS, WEIGHT_PER_GAS, extractWeight, getBlockArray } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import type { GenericExtrinsic } from "@polkadot/types";
import type { FrameSystemEventRecord } from "@polkadot/types/lookup";
import type { AnyTuple } from "@polkadot/types/types";
import { rateLimiter } from "../../helpers/common.js";

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : THIRTY_MINS;
const timeout = Math.floor(timePeriod / 12); // 2 hour -> 10 minute timeout
const limiter = rateLimiter();

interface BlockInfo {
  blockNum: number;
  hash: string;
  weights: {
    normal: bigint;
    operational: bigint;
    mandatory: bigint;
  };
  extrinsics: GenericExtrinsic<AnyTuple>[];
  events: FrameSystemEventRecord[];
}

interface BlockLimits {
  normal: bigint;
  operational: bigint;
}

describeSuite({
  id: "S05",
  foundationMethods: "read_only",
  title:
    "Verifying weights of blocks in the past " +
    `${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours`,

  testCases: ({ context, it, log }) => {
    let blockLimits: BlockLimits;
    let blockInfoArray: BlockInfo[];
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para") as any as ApiPromise;
      const blockNumArray = await getBlockArray(paraApi, timePeriod);
      const limits = paraApi.consts.system.blockWeights;

      const getLimits = async (blockNum: number) => {
        const blockHash = await paraApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await paraApi.at(blockHash);
        const {
          block: { extrinsics },
        } = await paraApi.rpc.chain.getBlock(blockHash);
        const specVersion = apiAt.consts.system.version.specVersion.toNumber();
        const events = await apiAt.query.system.events();
        if (specVersion >= 1700) {
          // TODO: replace type when we update to use SpWeightsWeightV2Weight
          const { normal, operational, mandatory } = await apiAt.query.system.blockWeight();
          return {
            blockNum,
            hash: blockHash.toString(),
            weights: {
              normal: extractWeight(normal).toBigInt(),
              operational: extractWeight(operational).toBigInt(),
              mandatory: extractWeight(mandatory).toBigInt(),
            },
            events,
            extrinsics,
          };
        }
      };

      // Support for weight v1 and weight v2.
      blockLimits = {
        normal: extractWeight(limits.perClass.normal.maxTotal).toBigInt(),
        operational: extractWeight(limits.perClass.operational.maxTotal).toBigInt(),
      };
      blockInfoArray = (await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getLimits(num)))
      )) as BlockInfo[];
    }, timeout);

    // This test is more for verifying that the test code is correctly returning good quality data
    // that the rest of the test suite performs verification on
    it({
      id: "C100",
      title: `should be returning unique block hashes in array`,
      test: async () => {
        const hashes = blockInfoArray.map((a) => a.hash);
        const set = new Set(hashes);
        expect(hashes.length, "Duplicate hashes in retrieved data, investigate test").to.be.equal(
          set.size
        );
      },
    });

    // Normal class
    it({
      id: "C200",
      title: "normal usage should be less than normal dispatch class limits",
      test: async function () {
        const overweight = blockInfoArray
          .filter((a) => a.weights.normal > blockLimits.normal)
          .map((a) => {
            log(
              `Block #${a.blockNum} has weight ${Number(a.weights.normal)} which is above limit!`
            );
            return a;
          });
        expect(
          overweight,
          `These blocks have normal weights in excess of limit, investigate: ${overweight
            .map((a) => a.blockNum)
            .join(", ")}`
        ).to.be.empty;
      },
    });

    // Operational class
    it({
      id: "C300",
      title: "operational usage should be less than dispatch class limits",
      test: async function () {
        const overweight = blockInfoArray
          .filter((a) => a.weights.operational > blockLimits.operational)
          .map((a) => {
            log(
              `Block #${a.blockNum} has weight ${Number(
                a.weights.operational
              )} which is above limit!`
            );
            return a;
          });
        expect(
          overweight,
          `These blocks have operational weights in excess of limit, investigate: ${overweight
            .map((a) => a.blockNum)
            .join(", ")}`
        ).to.be.empty;
      },
    });

    // This will test that when Block is 20%+ full, its normal weight is mostly explained
    // by eth signed transactions.
    it({
      id: "C400",
      title: "should roughly have a block weight mostly composed of transactions",
      timeout,
      test: async function () {
        // Waiting for bugfixes
        if (paraApi.consts.system.version.specVersion.toNumber() < 2000) {
          return; // TODO: replace this with skip() when added to vitest
        }

        log(
          `Checking #${blockInfoArray[0].blockNum} - #${
            blockInfoArray[blockInfoArray.length - 1].blockNum
          } block weight proportions.`
        );

        const checkBlockWeight = async (blockInfo: BlockInfo) => {
          const apiAt = await paraApi.at(blockInfo.hash);
          const normalWeight = blockInfo.weights.normal;
          const maxWeight = blockLimits.normal;
          const ethBlock = (await apiAt.query.ethereum.currentBlock()).unwrap();
          const balTxns = blockInfo.extrinsics
            .map((ext, index) =>
              ext.method.method === "transfer" && ext.method.section === "balances" ? index : -1
            )
            .filter((a) => a !== -1);
          const balTxnWeights = blockInfo.events
            .map((event) =>
              paraApi.events.system.ExtrinsicSuccess.is(event.event) &&
              event.phase.isApplyExtrinsic &&
              balTxns.includes(event.phase.asApplyExtrinsic.toNumber())
                ? event.event.data.dispatchInfo.weight.refTime.toBigInt()
                : 0n
            )
            .reduce((acc, curr) => acc + curr, 0n);

          const actualWeightUsed = (normalWeight * 100n) / maxWeight;
          if (actualWeightUsed > 20n) {
            const gasUsed = ethBlock.header.gasUsed.toBigInt();
            const weightCalc = gasUsed * WEIGHT_PER_GAS;
            const newRatio = ((normalWeight - weightCalc - balTxnWeights) * 100n) / maxWeight;
            if (newRatio > 20n) {
              log(
                `Block #${blockInfo.blockNum} is ${actualWeightUsed}% full with ` +
                  ethBlock.transactions.length +
                  ` transactions, non-transaction weight: ${newRatio}%`
              );
            }
            return { blockNum: blockInfo.blockNum, nonTxn: newRatio };
          }
        };

        const results = await Promise.all(
          blockInfoArray.map((blockInfo) => limiter.schedule(() => checkBlockWeight(blockInfo)))
        );
        const nonTxnHeavyBlocks = results.filter((a) => a && a.nonTxn > 20n);
        expect(
          nonTxnHeavyBlocks,
          `These blocks have non-txn weights >20%, please investigate: ${nonTxnHeavyBlocks
            .map((a) => a!.blockNum)
            .join(", ")}`
        ).to.be.empty;
      },
    });

    // This will test that the total normal weight reported is roughly the sum of normal class
    // weight events emitted by signed extrinsics
    it({
      id: "C500",
      title: `should have total normal weight matching the signed extrinsics`,
      timeout,
      test: async function () {
        // Waiting for bugfixes
        if (paraApi.consts.system.version.specVersion.toNumber() < 2000) {
          log("Skipping test due to RT ver < 2000 ");
          return; // TODO: replace this with skip() when added to vitest
        }

        const apiAt = await paraApi.at(blockInfoArray[0].hash);
        if (apiAt.consts.system.version.specVersion.toNumber() < 2000) {
          log("Skipping test due to RT ver < 2000 ");
          return;
        }

        log(
          `Checking if #${blockInfoArray[0].blockNum} - #${
            blockInfoArray[blockInfoArray.length - 1].blockNum
          } extrinsic weights sum up.`
        );

        const checkWeights = (blockInfo: BlockInfo) => {
          // Skip this block if substrate balance transfer ext, due to weight reporting
          if (
            blockInfo.extrinsics.find(
              (x) => x.method.method === "transfer" && x.method.section === "balances"
            )
          ) {
            return {
              blockNum: blockInfo.blockNum,
              signedExtTotal: -1,
              normalWeights: -1,
              difference: 0,
            };
          }

          const signedExtTotal = blockInfo.events
            .filter(
              (a) => a.event.method === "ExtrinsicSuccess" || a.event.method === "ExtrinsicFailed"
            )
            .filter((a) => (a.event.data as any).dispatchInfo.class.toString() === "Normal")
            .reduce(
              (acc, curr) =>
                acc + extractWeight((curr.event.data as any).dispatchInfo.weight).toNumber(),
              0
            );
          const normalWeights = Number(blockInfo.weights.normal);
          const difference = (normalWeights - signedExtTotal) / signedExtTotal;
          if (difference > 0.2) {
            log(
              `Block #${blockInfo.blockNum} signed extrinsic weight - reported: ${signedExtTotal}, 
            accounted: ${normalWeights} (${difference > 0 ? "+" : "-"}${(difference * 100).toFixed(
              2
            )}%).`
            );
          }
          return { blockNum: blockInfo.blockNum, signedExtTotal, normalWeights, difference };
        };

        const results = blockInfoArray.map((blockInfo) => checkWeights(blockInfo));
        const heavyweights = results.filter((a) => a.difference > 0.2);
        expect(
          heavyweights,
          `These blocks have >20% overweight normal weights, please investigate: ${heavyweights
            .map((a) => a.blockNum)
            .join(", ")}`
        ).to.be.empty;
      },
    });

    // This test will compare the total weight of eth transactions versus the reported gasUsed
    // property of  ethereum.currentBlock()
    it({
      id: "C600",
      title: "should have total gas charged similar to eth extrinsics",
      timeout,
      test: async function () {
        // Waiting for bugfixes
        if (paraApi.consts.system.version.specVersion.toNumber() < 2000) {
          log("Skipping test due to RT ver < 2000 ");
          return;
        }

        log(
          `Checking if #${blockInfoArray[0].blockNum} - #${
            blockInfoArray[blockInfoArray.length - 1].blockNum
          } weights match gasUsed`
        );

        const compareGasToWeight = async (blockInfo: BlockInfo) => {
          const apiAt = await paraApi.at(blockInfo.hash);
          const signedBlock = await paraApi.rpc.chain.getBlock(blockInfo.hash);
          const gasUsed = (await apiAt.query.ethereum.currentBlock())
            .unwrap()
            .header.gasUsed.toNumber();

          const gasWeight = gasUsed * Number(WEIGHT_PER_GAS);
          const ethTxnsWeight = signedBlock.block.extrinsics
            .map((item, index) => {
              if (item.method.method === "transact" && item.method.section === "ethereum") {
                return blockInfo.events
                  .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
                  .filter(
                    ({ event }) => event.method === "ExtrinsicSuccess" && event.section === "system"
                  )
                  .reduce(
                    (acc, curr) =>
                      acc + extractWeight((curr.event.data as any).dispatchInfo.weight).toNumber(),
                    0
                  );
              }
              return 0;
            })
            .reduce((acc, curr) => acc + curr, 0);
          const difference = ethTxnsWeight - gasWeight;

          if (difference > 0) {
            log(
              `Block #${blockInfo.blockNum} has a ${((difference / ethTxnsWeight) * 100).toFixed(
                2
              )}% discrepancy between eth gas used and weight charged. `
            );
          }
          return { blockNum: blockInfo.blockNum, gasWeight, ethTxnsWeight, difference };
        };

        const results = await Promise.all(
          blockInfoArray.map((blockInfo) => limiter.schedule(() => compareGasToWeight(blockInfo)))
        );
        const discrepancies = results.filter((a) => a.difference > 0);
        expect(
          discrepancies,
          `These blocks have mismatching gas used vs charged weight, 
        please investigate: ${discrepancies.map((a) => a.blockNum).join(", ")}`
        ).to.be.empty;
      },
    });
  },
});
