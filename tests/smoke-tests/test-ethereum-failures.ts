import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import type { DispatchInfo } from "@polkadot/types/interfaces";
import Bottleneck from "bottleneck";
import { FrameSystemEvent, FrameSystemEventRecord } from "@polkadot/types/lookup";
const debug = require("debug")("smoke:eth-failures");

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.max(Math.floor(timePeriod / 12), 5000);
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });

type BlockEventsRecord = {
  blockNum: number;
  events: FrameSystemEventRecord[];
};

describeSmokeSuite(
  `ETH Failures in past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours` +
    " should be charged correctly...",
  (context) => {
    let blockEvents: BlockEventsRecord[];

    before("Retrieve events for previous blocks", async function () {
      this.timeout(timeout);

      const blockNumArray = await getBlockArray(context.polkadotApi, timePeriod, limiter);

      const getEvents = async (blockNum: number) => {
        const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await context.polkadotApi.at(blockHash);
        const events = await apiAt.query.system.events();
        return { blockNum, events };
      };

      blockEvents = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getEvents(num)))
      );
    });

    it("successful exts should always pays_fee: no", async function () {
      this.timeout(timeout);
      const filteredEvents = blockEvents
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

      const isEthereumTxn = async (blockNum: number, index: number) => {
        const hash = await limiter.schedule(() =>
          context.polkadotApi.rpc.chain.getBlockHash(blockNum)
        );
        const signedBlock = await limiter.schedule(() =>
          context.polkadotApi.rpc.chain.getBlock(hash)
        );
        return (
          signedBlock.block.extrinsics[index].method.section.toString() === "ethereum" &&
          signedBlock.block.extrinsics[index].method.method.toString() === "transact"
        );
      };

      const ethFilteredEvents = filteredEvents.map(async ({ blockNum, matchedEvents }) => {
        // const ethEvents = (await Promise.all(matchedEvents.map(async (a) => {
        //     const result = await isEthereumTxn(blockNum, a.phase.asApplyExtrinsic.toNumber());
        //     if (result) {
        //       return a;
        //     } else {
        //       return []
        //     }
        // }))).filter((a: any)=>a.length >0)
        const ethEvents = await Promise.all(
          matchedEvents.filter(async (a) => {
            return await isEthereumTxn(blockNum, a.phase.asApplyExtrinsic.toNumber());
          })
        );
        return { blockNum, matchedEvents: ethEvents };
      });

      const results = await Promise.all(ethFilteredEvents);

      const failures = results.filter((a) => a.matchedEvents.length > 0);
      console.log(failures);
      failures.forEach(({ blockNum, matchedEvents }) => {
        matchedEvents.forEach((a: any) => {
          debug(
            `ETH txn at block #${blockNum} extrinsic #${a.phase.asApplyExtrinsic.toNumber()}: pays_fee = Yes`
          );
        });
      });

      /// TODO: do more testing if this works for isNo and other stuff

      expect(
        failures.length,
        `pays_fee:yes in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });
  }
);

/// TODO ADD THE OTHER CASES WE NEED TO FILTER FOR
