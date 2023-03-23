import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { checkTimeSliceForUpgrades, getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import type { DispatchInfo } from "@polkadot/types/interfaces";
import Bottleneck from "bottleneck";
import { FrameSystemEventRecord } from "@polkadot/types/lookup";
import { GenericExtrinsic } from "@polkadot/types";
import { AnyTuple } from "@polkadot/types/types";
const debug = require("debug")("smoke:eth-failures");
const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.max(Math.floor(timePeriod / 12), 5000);
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
const hours = (timePeriod / (1000 * 60 * 60)).toFixed(2);

type BlockFilteredRecord = {
  blockNum: number;
  extrinsics: GenericExtrinsic<AnyTuple>[];
  events: FrameSystemEventRecord[];
  ethTxns;
  receipts;
};

describeSmokeSuite(
  "S900",
  `ETH Failures in past ${hours} hours should be reported correctly`,

  (context, testIt) => {
    let blockData: BlockFilteredRecord[];

    before("Retrieve events for previous blocks", async function () {
      this.timeout(timeout);
      const blockNumArray = await getBlockArray(context.polkadotApi, timePeriod, limiter);

      debug(`Collecting ${hours} hours worth of events`);

      const getBlockData = async (blockNum: number) => {
        const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
        const apiAt = await context.polkadotApi.at(blockHash);
        return {
          blockNum: blockNum,
          extrinsics: signedBlock.block.extrinsics,
          events: await apiAt.query.system.events(),
          ethTxns: (await apiAt.query.ethereum.currentTransactionStatuses()).unwrap(),
          receipts: (await apiAt.query.ethereum.currentReceipts()).unwrap(),
        };
      };

      // Determine if the block range intersects with an upgrade event
      const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
        context.polkadotApi,
        blockNumArray,
        context.polkadotApi.consts.system.version.specVersion
      );
      if (result) {
        debug(`Time slice of blocks intersects with upgrade from RT ${onChainRt}, skipping tests.`);
        this.skip();
      }

      blockData = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getBlockData(num)))
      );
    });

    /// This test will check that all ethereum.transact extrinsics have a corresponding
    /// paysFee = no property in ExtrinsicSuccess event
    testIt("C100", `successful eth exts should always pays_fee: no`, function () {
      this.timeout(30000);
      const filteredEvents = blockData
        .map(({ blockNum, events }) => {
          const matchedEvents = events
            .filter(({ event }) => context.polkadotApi.events.system.ExtrinsicSuccess.is(event))
            .filter(({ event }) => {
              const info = event.data[0] as DispatchInfo;
              return info.class.isNormal && info.paysFee.isYes;
            });
          return { blockNum, matchedEvents };
        })
        .filter(({ matchedEvents }) => matchedEvents.length > 0);

      const isEthereumTxn = (blockNum: number, index: number) => {
        const extrinsic = blockData.find((a) => a.blockNum === blockNum).extrinsics[index];
        return (
          extrinsic.method.section.toString() === "ethereum" &&
          extrinsic.method.method.toString() === "transact"
        );
      };

      const failures = filteredEvents
        .map(({ blockNum, matchedEvents }) => {
          const ethEvents = matchedEvents.filter((a) =>
            isEthereumTxn(blockNum, a.phase.asApplyExtrinsic.toNumber())
          );
          return { blockNum, matchedEvents: ethEvents };
        })
        .filter((a) => a.matchedEvents.length > 0);

      failures.forEach(({ blockNum, matchedEvents }) => {
        matchedEvents.forEach((a: any) => {
          debug(
            `ETH txn at block #${blockNum} extrinsic #${a.phase.asApplyExtrinsic.toNumber()}` +
              ": pays_fee = Yes"
          );
        });
      });

      expect(
        failures.length,
        `Please investigate blocks ${failures.map((a) => a.blockNum).join(`, `)}; pays_fee:yes  `
      ).to.equal(0);
    });

    // This test will check that each ethereum.transact extrinsic has a corresponding event
    // of ExtrinsicSuccess fired. Any Extrinsic.Failed events will be reported and mark the
    // block for further investigation.
    testIt("C200", `should have have ExtrinsicSuccess for all ethereum.transact`, function () {
      this.timeout(30000);
      debug(
        `Checking ${blockData.reduce((curr, acc) => curr + acc.extrinsics.length, 0)}` +
          " eth extrinsics all have corresponding ExtrinsicSuccess events."
      );
      const blockWithFailures = blockData
        .map(({ blockNum, extrinsics, events }) => {
          const successes = extrinsics
            .map((item, index) => {
              if (
                item.method.section.toString() === "ethereum" &&
                item.method.method.toString() === "transact"
              ) {
                const success = events
                  .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
                  .find(({ event, phase }) => {
                    if (context.polkadotApi.events.system.ExtrinsicFailed.is(event)) {
                      debug(
                        `ethereum.transact has ExtrinsicFailed event - Block: ${blockNum}` +
                          " extrinsic: " +
                          phase.asApplyExtrinsic.toNumber() +
                          `.`
                      );
                    }
                    return context.polkadotApi.events.system.ExtrinsicSuccess.is(event);
                  });

                if (success) {
                  return true;
                } else {
                  return false;
                }
              }
              return undefined;
            })
            .filter((a) => typeof a !== "undefined")
            .reduce((acc, curr) => curr && acc, true);
          return { blockNum, successes };
        })
        .filter((a) => a.successes === false);

      expect(
        blockWithFailures.length,
        `Please investigate blocks ${blockWithFailures.map((a) => a.blockNum).join(`, `)}`
      ).to.equal(0);
    });

    testIt(
      "C300",
      `should have matching amounts in emulated block as there are ethereum.executed events`,
      function () {
        this.timeout(30000);
        const ethEvents = blockData.map(({ blockNum, events, ethTxns }) => {
          const successes = events.filter(({ event }) =>
            context.polkadotApi.events.ethereum.Executed.is(event)
          );
          return { blockNum, ethEvents: successes.length, ethTxns: ethTxns.length };
        });

        const failures = ethEvents.filter((a) => a.ethEvents !== a.ethTxns);
        failures.forEach((a) =>
          debug(
            `Block #${a.blockNum} has mismatching amounts - ` +
              `${a.ethEvents} eth extrinsics vs ` +
              `${a.ethTxns} eth txns.`
          )
        );

        expect(
          failures.length,
          `Accepted ETH transactions do not match submitted ETH extrinsics for blocks: ${failures
            .map((a) => a.blockNum)
            .join(`, `)}`
        ).to.equal(0);
      }
    );

    testIt(
      "C400",
      `should have a receipt in emulated block for each ethereum.executed event`,
      function () {
        this.timeout(30000);
        const ethEvents = blockData.map(({ blockNum, events, ethTxns }) => {
          const successes = events.filter(({ event }) =>
            context.polkadotApi.events.ethereum.Executed.is(event)
          );
          return { blockNum, ethEvents: successes.length, ethReceipts: ethTxns.length };
        });

        const failures = ethEvents.filter((a) => a.ethEvents !== a.ethReceipts);
        failures.forEach((a) =>
          debug(
            `Block #${a.blockNum} has mismatching amounts - ` +
              `${a.ethEvents} eth extrinsics vs ` +
              `${a.ethReceipts} eth receipts.`
          )
        );

        expect(
          failures.length,
          `Accepted ETH transactions do not match submitted ETH extrinsics for blocks: ${failures
            .map((a) => a.blockNum)
            .join(`, `)}`
        ).to.equal(0);
      }
    );
  }
);
