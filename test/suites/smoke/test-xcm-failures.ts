import "@moonbeam-network/api-augment/moonbase";
import { rateLimiter, checkTimeSliceForUpgrades } from "../../helpers/common.js";
import type { FrameSystemEventRecord, XcmV3MultiLocation } from "@polkadot/types/lookup";
import {
  type MoonbeamNetworkName,
  type ParaId,
  isMuted,
  ForeignChainsEndpoints,
} from "../../helpers/foreign-chains.js";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { getBlockArray, FIVE_MINS, ONE_HOURS } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : ONE_HOURS;
const atBlock = process.env.AT_BLOCK ? Number(process.env.AT_BLOCK) : -1;
const timeout = Math.max(Math.floor(timePeriod / 12), 5000);
const limiter = rateLimiter();

type BlockEventsRecord = {
  blockNum: number;
  events: FrameSystemEventRecord[];
};

describeSuite({
  id: "S25",
  title:
    `XCM Failures in past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours` +
    ` should not be serious`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let blockEvents: BlockEventsRecord[];
    let chainName: MoonbeamNetworkName;
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;
    let isUpgrading: boolean;
    let aboveRt2900: boolean;
    let networkSkip: boolean;

    const isMutedChain = (events: FrameSystemEventRecord[], index: number) => {
      let muted = false;
      if (paraApi.events.polkadotXcm.AssetsTrapped.is(events[Math.max(0, index - 1)].event)) {
        const { interior } = events[index - 1].event.data[1] as XcmV3MultiLocation;
        if (interior.isX1) {
          muted = !!isMuted(chainName, interior.asX1.asParachain.toNumber() as ParaId);
        }
      }
      return muted;
    };

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      relayApi = context.polkadotJs("relay");

      const blockNumArray = atBlock > 0 ? [atBlock] : await getBlockArray(paraApi, timePeriod);

      // Determine if this block range intersects with an upgrade event
      const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
        paraApi,
        blockNumArray,
        paraApi.consts.system.version.specVersion
      );

      // PolkadotSDK 1.7.2 removes XCM errors, so we can skip these tests
      aboveRt2900 = onChainRt.toNumber() >= 2900;

      if (result) {
        log(
          `Time slice of blocks intersects with upgrade from RT ${onChainRt}, skipping all tests.`
        );
        isUpgrading = true;
        return;
      }

      const chainQuery = (await paraApi.rpc.system.chain()).toString();

      if (!ForeignChainsEndpoints.find((chain) => chain.moonbeamNetworkName === chainQuery)) {
        log(
          `Non-prod chains have unreliable channels, skipping HRMP monitoring for ${chainQuery}.`
        );
        networkSkip = true;
      }

      chainName = (await paraApi.rpc.system.chain()).toString() as MoonbeamNetworkName;

      const getEvents = async (blockNum: number) => {
        const blockHash = await paraApi.rpc.chain.getBlockHash(blockNum);
        const apiAt = await paraApi.at(blockHash);
        const events = await apiAt.query.system.events();
        return { blockNum, events };
      };

      blockEvents = await Promise.all(
        blockNumArray.map((num) => limiter.schedule(() => getEvents(num)))
      );
    }, timeout);

    it({
      id: "C100",
      title: "should not have UnsupportedVersion errors on cumulusXcm queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const dmpQueueEvents = events.filter(
            ({ event }, idx) =>
              paraApi.events.dmpQueue.UnsupportedVersion.is(event) && !isMutedChain(events, idx)
          );
          return { blockNum, dmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.dmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM error dmpQueue.UnsupportedVersion in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C200",
      title: "should not have BadVersion errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events.filter(
            ({ event }, idx) =>
              paraApi.events.xcmpQueue.BadVersion.is(event) && !isMutedChain(events, idx)
          );
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) => log(`XCM error xcmpQueue.BadVersion in block #${a.blockNum}.`));
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C300",
      title: "should not have Barrier errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "Barrier");
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) => log(`XCM Barrier error xcmpQueue.Fail in block #${a.blockNum}.`));
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C400",
      title: "should not have Overflow errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "Overflow");
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) => log(`XCM Overflow error xcmpQueue.Fail in block #${a.blockNum}.`));
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C500",
      title: "should not have MultiLocationFull errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "MultiLocationFull"
            );
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM MultiLocationFull error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C600",
      title: "should not have AssetNotFound errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "AssetNotFound");
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM AssetNotFound error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C700",
      title: "should not have DestinationUnsupported errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "DestinationUnsupported"
            );
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM DestinationUnsupported error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C800",
      title: "should not have Transport errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "Transport");
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) => log(`XCM Transport error xcmpQueue.Fail in block #${a.blockNum}.`));
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C900",
      title: "should not have FailedToDecode errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "FailedToDecode");
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM FailedToDecode error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1000",
      title: "should not have UnhandledXcmVersion errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "UnhandledXcmVersion"
            );
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM UnhandledXcmVersion error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1100",
      title: "should not have WeightNotComputable errors on XCMP queue",
      test: async function (context) {
        if (isUpgrading || aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.xcmpQueue.Fail.is(event) && !isMutedChain(events, idx)
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "WeightNotComputable"
            );
          return { blockNum, xcmpQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM WeightNotComputable error xcmpQueue.Fail in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1200",
      title: "should have recent responses for opened HMRP channels",
      timeout: FIVE_MINS,
      test: async function (context) {
        if (isUpgrading || networkSkip) {
          context.skip();
        }

        const paraId = await paraApi.query.parachainInfo.parachainId();
        const inChannels = (
          (await relayApi.query.hrmp.hrmpIngressChannelsIndex(paraId)) as any
        ).map((a: any) => a.toNumber());
        const outChannels = (
          (await relayApi.query.hrmp.hrmpEgressChannelsIndex(paraId)) as any
        ).map((a: any) => a.toNumber());
        const channels = [...new Set([...inChannels, ...outChannels])];

        const fiveMinutesOfBlocks = await getBlockArray(relayApi, FIVE_MINS);

        const getEvents = async (blockNum: number) => {
          const blockHash = await relayApi.rpc.chain.getBlockHash(blockNum);
          const apiAt = await relayApi.at(blockHash);
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
          log(`No response in 5 minutes for connected Parachain #${a.channel}`)
        );
        expect(
          failedResponses.length,
          `Open channels exist with unresponsive chains: ${failedResponses
            .map((a) => a.channel)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1300",
      title: "should not have OverweightEnqueued errors on message queue",
      minRtVersion: 2900,
      test: async function (context) {
        if (isUpgrading || !aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const messageQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.messageQueue.OverweightEnqueued.is(event) &&
                !isMutedChain(events, idx)
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "OverweightEnqueued"
            );
          return { blockNum, messageQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.messageQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM OverweightEnqueued error messageQueue in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1400",
      title: "should not have ProcessingFailed errors on message queue",
      minRtVersion: 2900,
      test: async function (context) {
        if (isUpgrading || !aboveRt2900) {
          context.skip();
        }
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const messageQueueEvents = events
            .filter(
              ({ event }, idx) =>
                paraApi.events.messageQueue.ProcessingFailed.is(event) && !isMutedChain(events, idx)
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "ProcessingFailed");
          return { blockNum, messageQueueEvents };
        });

        const failures = filteredEvents.filter((a) => a.messageQueueEvents.length !== 0);
        failures.forEach((a) =>
          log(`XCM ProcessingFailed error messageQueue in block #${a.blockNum}.`)
        );
        expect(
          failures.length,
          `XCM errors in blocks ${failures.map((a) => a.blockNum).join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });
  },
});
