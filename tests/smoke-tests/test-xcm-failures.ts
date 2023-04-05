import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { checkTimeSliceForUpgrades, getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { FrameSystemEventRecord, XcmV1MultiLocation } from "@polkadot/types/lookup";
import { FIVE_MINS } from "../util/constants";
import { isMuted } from "../util/foreign-chains";
const debug = require("debug")("smoke:xcm-failures");

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 2 * 60 * 60 * 1000;
const atBlock = process.env.AT_BLOCK ? Number(process.env.AT_BLOCK) : -1;
const timeout = Math.max(Math.floor(timePeriod / 12), 5000);
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });

type BlockEventsRecord = {
  blockNum: number;
  events: FrameSystemEventRecord[];
};

describeSmokeSuite(
  "S2300",
  `XCM Failures in past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours` +
    ` should not be serious`,

  (context, testIt) => {
    let blockEvents: BlockEventsRecord[];
    let chainName: string;

    const isMutedChain = (events: FrameSystemEventRecord[], index: number) => {
      let muted = false;
      if (
        context.polkadotApi.events.polkadotXcm.AssetsTrapped.is(
          events[Math.max(0, index - 1)].event
        )
      ) {
        const { interior } = events[index - 1].event.data[1] as XcmV1MultiLocation;
        if (interior.isX1) {
          muted = isMuted(chainName, interior.asX1.asParachain.toNumber());
        }
      }
      return muted;
    };

    before("Retrieve events for previous blocks", async function () {
      this.timeout(timeout);

      const blockNumArray =
        atBlock > 0 ? [atBlock] : await getBlockArray(context.polkadotApi, timePeriod, limiter);

      // Determine if this block range intersects with an upgrade event
      const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
        context.polkadotApi,
        blockNumArray,
        context.polkadotApi.consts.system.version.specVersion
      );
      if (result) {
        debug(`Time slice of blocks intersects with upgrade from RT ${onChainRt}, skipping tests.`);
        this.skip();
      }

      chainName = (await context.polkadotApi.rpc.system.chain()).toString();

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

    testIt("C100", `should not have UnsupportedVersion errors on DMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const dmpQueueEvents = events.filter(
          ({ event }, idx) =>
            context.polkadotApi.events.dmpQueue.UnsupportedVersion.is(event) &&
            !isMutedChain(events, idx)
        );
        return { blockNum, dmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.dmpQueueEvents.length !== 0);
      failures.forEach((a) =>
        debug(`XCM error dmpQueue.UnsupportedVersion in block #${a.blockNum}.`)
      );
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C200", `should not have BadVersion errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events.filter(
          ({ event }, idx) =>
            context.polkadotApi.events.xcmpQueue.BadVersion.is(event) && !isMutedChain(events, idx)
        );
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) => debug(`XCM error xcmpQueue.BadVersion in block #${a.blockNum}.`));
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C300", `should not have Barrier errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(({ event: { data } }) => (data as any).error.toString() === "Barrier");
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) => debug(`XCM Barrier error xcmpQueue.Fail in block #${a.blockNum}.`));
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C400", `should not have Overflow errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(({ event: { data } }) => (data as any).error.toString() === "Overflow");
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) => debug(`XCM Overflow error xcmpQueue.Fail in block #${a.blockNum}.`));
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C500", `should not have MultiLocationFull errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(({ event: { data } }) => (data as any).error.toString() === "MultiLocationFull");
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) =>
        debug(`XCM MultiLocationFull error xcmpQueue.Fail in block #${a.blockNum}.`)
      );
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C600", `should not have AssetNotFound errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(({ event: { data } }) => (data as any).error.toString() === "AssetNotFound");
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) =>
        debug(`XCM AssetNotFound error xcmpQueue.Fail in block #${a.blockNum}.`)
      );
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt(
      "C700",
      `should not have DestinationUnsupported errors on XCMP queue`,
      async function () {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "DestinationUnsupported"
            );
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          debug(`XCM DestinationUnsupported error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      }
    );

    testIt("C800", `should not have Transport errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(({ event: { data } }) => (data as any).error.toString() === "Transport");
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) => debug(`XCM Transport error xcmpQueue.Fail in block #${a.blockNum}.`));
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C900", `should not have FailedToDecode errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(({ event: { data } }) => (data as any).error.toString() === "FailedToDecode");
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) =>
        debug(`XCM FailedToDecode error xcmpQueue.Fail in block #${a.blockNum}.`)
      );
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C1000", `should not have UnhandledXcmVersion errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(
            ({ event: { data } }) => (data as any).error.toString() === "UnhandledXcmVersion"
          );
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) =>
        debug(`XCM UnhandledXcmVersion error xcmpQueue.Fail in block #${a.blockNum}.`)
      );
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C1100", `should not have WeightNotComputable errors on XCMP queue`, async function () {
      const filteredEvents = blockEvents.map(({ blockNum, events }) => {
        const xcmpQueueEvents = events
          .filter(
            ({ event }, idx) =>
              context.polkadotApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
          )
          .filter(
            ({ event: { data } }) => (data as any).error.toString() === "WeightNotComputable"
          );
        return { blockNum, xcmpQueueEvents };
      });

      const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
      failures.forEach((a) =>
        debug(`XCM WeightNotComputable error xcmpQueue.Fail in block #${a.blockNum}.`)
      );
      expect(
        failures.length,
        `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C1200", `should have recent responses for opened HMRP channels`, async function () {
      this.timeout(FIVE_MINS);
      if (typeof process.env.RELAY_WSS_URL === "undefined" || process.env.RELAY_WSS_URL === "") {
        debug(`RELAY_WSS_URL env var not supplied, skipping test.`);
        this.skip();
      }

      if (chainName !== "Moonbeam" && chainName !== "Moonriver") {
        debug(`Non-prod chains have unreliable channels, skipping test for ${chainName}.`);
        this.skip();
      }

      const paraId = await context.polkadotApi.query.parachainInfo.parachainId();
      const inChannels = (
        (await context.relayApi.query.hrmp.hrmpIngressChannelsIndex(paraId)) as any
      ).map((a) => a.toNumber());
      const outChannels = (
        (await context.relayApi.query.hrmp.hrmpIngressChannelsIndex(paraId)) as any
      ).map((a) => a.toNumber());
      const channels = [...new Set([...inChannels, ...outChannels])];

      const fiveMinutesOfBlocks = await getBlockArray(context.relayApi, FIVE_MINS, limiter);

      const getEvents = async (blockNum: number) => {
        const blockHash = await context.relayApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await context.relayApi.at(blockHash);
        const events = await apiAt.query.system.events();
        return { blockNum, events };
      };

      const fiveMinutesOfEvents = await Promise.all(
        fiveMinutesOfBlocks.map((num) => limiter.schedule(() => getEvents(num)))
      );

      const responses = channels
        .filter((a) => !isMuted(chainName, a))
        .map((channel) => {
          const record = fiveMinutesOfEvents.find(({ events }) => {
            const matchedEvent = events
              .filter(
                ({ event }) =>
                  event.method.toString() === "CandidateIncluded" &&
                  event.section.toString() === "paraInclusion"
              )
              .find(({ event: { data } }) => {
                const {
                  descriptor: { paraId },
                } = data[0] as any;
                return paraId.toNumber() === channel;
              });
            return typeof matchedEvent !== "undefined";
          });
          const response = typeof record !== "undefined";
          return { channel, response };
        });
      const failedResponses = responses.filter((a) => a.response === false);
      failedResponses.forEach((a) =>
        debug(`No response in 5 minutes for connected Parachain #${a.channel}`)
      );
      expect(
        failedResponses.length,
        `Open channels exist with unresponsive chains: ${failedResponses
          .map((a) => a.channel)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });
  }
);
