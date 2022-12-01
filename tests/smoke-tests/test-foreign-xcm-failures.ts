import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { FrameSystemEventRecord } from "@polkadot/types/lookup";
import { FIVE_MINS } from "../util/constants";
import { ForeignChainsEndpoints } from "../util/foreign-chains";
import { ApiPromise, WsProvider } from "@polkadot/api";
const debug = require("debug")("smoke:foreign-xcm-fails");

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const timeout = Math.max(Math.floor(timePeriod / 12), 30000);
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });

type BlockEventsRecord = {
  blockNum: number;
  events: FrameSystemEventRecord[];
};

type NetworkBlockEvents = {
  networkName: string;
  blockEvents: BlockEventsRecord[];
};

describeSmokeSuite(
  `Foreign XCM Failures in past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours` +
    " should not be serious..",
  (context) => {
    let networkBlockEvents: NetworkBlockEvents[];

    before("Retrieve events for previous blocks", async function () {
      const networkName = context.polkadotApi.runtimeChain.toString();
      const foreignChainInfos = ForeignChainsEndpoints.find(
        (a) => a.moonbeamNetworkName === networkName
      );

      if (foreignChainInfos == null) {
        debug(`No Foreign chain endpoints available for network ${networkName}, skipping.`);
        this.skip();
      }
      this.timeout(timeout * foreignChainInfos.foreignChains.length);

      const promises = foreignChainInfos.foreignChains.map(async ({ name, endpoints }) => {
        let result: NetworkBlockEvents;
        try {
          const api = await ApiPromise.create({
            provider: new WsProvider(endpoints),
            noInitWarn: true,
          });
          const blockNumArray = await getBlockArray(api, timePeriod, limiter);

          const getEvents = async (blockNum: number) => {
            const blockHash = await limiter.schedule(() => api.rpc.chain.getBlockHash(blockNum));
            const apiAt = await limiter.schedule(() => api.at(blockHash));
            const events = await limiter.schedule(() => apiAt.query.system.events());
            return { blockNum, events };
          };

          const blockEvents: BlockEventsRecord[] = await Promise.all(
            blockNumArray.map((num) => getEvents(num))
          );
          api.disconnect();
          result = { networkName: name, blockEvents };
        } catch (e) {
          debug(e);
          result = { networkName: name, blockEvents: [] };
        }
        return result;
      });
      networkBlockEvents = await Promise.all(promises);
    });

    it.only("should not have UnsupportedVersion errors on DMP queue", async function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const dmpQueueEvents = events.filter(
            ({ event }) =>
              event.section.toString() === "dmpQueue" &&
              // event.method.toString() === "UnsupportedVersion"
              event.method.toString() === "ExecutedDownward"
          );
          return { blockNum, dmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });
      console.log(blockEvents)

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.dmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            `XCM error dmpQueue.UnsupportedVersion in network ${networkName} block #${blockNum}.`
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    // it("should not have BadVersion errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events.filter(
    //       ({ event }) =>
    //         event.section.toString() === "xcmpQueue" && event.method.toString() === "BadVersion"
    //     );
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) => debug(`XCM error xcmpQueue.BadVersion in block #${a.blockNum}.`));
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have Barrier errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(({ event: { data } }) => (data as any).error.toString() === "Barrier");
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) => debug(`XCM Barrier error xcmpQueue.Fail in block #${a.blockNum}.`));
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have Overflow errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(({ event: { data } }) => (data as any).error.toString() === "Overflow");
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) => debug(`XCM Overflow error xcmpQueue.Fail in block #${a.blockNum}.`));
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have MultiLocationFull errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(({ event: { data } }) => (data as any).error.toString() === "MultiLocationFull");
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) =>
    //     debug(`XCM MultiLocationFull error xcmpQueue.Fail in block #${a.blockNum}.`)
    //   );
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have AssetNotFound errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(({ event: { data } }) => (data as any).error.toString() === "AssetNotFound");
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) =>
    //     debug(`XCM AssetNotFound error xcmpQueue.Fail in block #${a.blockNum}.`)
    //   );
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have DestinationUnsupported errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(
    //         ({ event: { data } }) => (data as any).error.toString() === "DestinationUnsupported"
    //       );
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) =>
    //     debug(`XCM DestinationUnsupported error xcmpQueue.Fail in block #${a.blockNum}.`)
    //   );
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have Transport errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(({ event: { data } }) => (data as any).error.toString() === "Transport");
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) => debug(`XCM Transport error xcmpQueue.Fail in block #${a.blockNum}.`));
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have FailedToDecode errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(({ event: { data } }) => (data as any).error.toString() === "FailedToDecode");
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) =>
    //     debug(`XCM FailedToDecode error xcmpQueue.Fail in block #${a.blockNum}.`)
    //   );
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have UnhandledXcmVersion errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(
    //         ({ event: { data } }) => (data as any).error.toString() === "UnhandledXcmVersion"
    //       );
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) =>
    //     debug(`XCM UnhandledXcmVersion error xcmpQueue.Fail in block #${a.blockNum}.`)
    //   );
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });

    // it("should not have WeightNotComputable errors on XCMP queue", async function () {
    //   const filteredEvents = blockEvents.map(({ blockNum, events }) => {
    //     const xcmpQueueEvents = events
    //       .filter(
    //         ({ event }) =>
    //           event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
    //       )
    //       .filter(
    //         ({ event: { data } }) => (data as any).error.toString() === "WeightNotComputable"
    //       );
    //     return { blockNum, xcmpQueueEvents };
    //   });

    //   const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
    //   failures.forEach((a) =>
    //     debug(`XCM WeightNotComputable error xcmpQueue.Fail in block #${a.blockNum}.`)
    //   );
    //   expect(
    //     failures.length,
    //     `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
    //   ).to.equal(0);
    // });
  }
);
