import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { checkTimeSliceForUpgrades, getBlockArray } from "../util/block";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import Bottleneck from "bottleneck";
import { FrameSystemEventRecord } from "@polkadot/types/lookup";
import { ForeignChainsEndpoints, getEndpoints } from "../util/foreign-chains";
import { ApiPromise, WsProvider } from "@polkadot/api";
const debug = require("debug")("smoke:foreign-xcm-fails");
const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : 30 * 60 * 1000;
const timeout = Math.max(Math.floor(timePeriod / 12), 60000);
const limiter = new Bottleneck({ maxConcurrent: 20 });

type BlockEventsRecord = {
  blockNum: number;
  events: FrameSystemEventRecord[];
};

type NetworkBlockEvents = {
  networkName: string;
  blockEvents: BlockEventsRecord[];
};

describeSmokeSuite(
  "S1100",
  `Foreign XCM Failures in past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours` +
    ` should not be serious`,

  (context, testIt) => {
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

      const relayName =
        networkName === "Moonbeam"
          ? "Polkadot"
          : networkName === "Moonriver"
          ? "Kusama"
          : "Unsupported";
      const chainsWithRpcs = foreignChainInfos.foreignChains.map((chain) => {
        const endpoints = getEndpoints(relayName, chain.paraId);
        return { ...chain, endpoints };
      });

      const promises = chainsWithRpcs.map(async ({ name, endpoints, mutedUntil }) => {
        let blockEvents: BlockEventsRecord[];

        if (mutedUntil >= new Date().getTime()) {
          debug(`Network tests for ${name} has been muted, skipping.`);
          return { networkName: name, blockEvents: [] };
        }

        try {
          const api: ApiPromise = await new Promise((resolve, reject) => {
            const provider = new WsProvider(endpoints);
            provider.on("connected", async () => {
              resolve(
                await ApiPromise.create({
                  provider,
                  noInitWarn: true,
                })
              );
            });
            provider.on("error", async () => {
              debug(`Could not connect to ${name}, skipping.`);
              provider.disconnect();
              reject();
            });
          });

          if (api == null) {
            throw new Error("Cannot Connect");
          }

          const blockNumArray = await getBlockArray(api, timePeriod, limiter);

          // Determine if the block range intersects with an upgrade event
          const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
            api,
            blockNumArray,
            api.consts.system.version.specVersion
          );
          if (result) {
            debug(
              `Time slice of blocks intersects with upgrade from RT ${onChainRt}, skipping chain.`
            );
            api.disconnect();
            return { networkName: name, blockEvents: [] };
          }

          const getEvents = async (blockNum: number) => {
            const blockHash = await limiter.schedule(() => api.rpc.chain.getBlockHash(blockNum));
            const apiAt = await limiter.schedule(() => api.at(blockHash));
            const events = await limiter.schedule(() => apiAt.query.system.events());
            return { blockNum, events };
          };

          blockEvents = await Promise.all(blockNumArray.map((num) => getEvents(num)));
          debug(`Finished loading blocks for ${name}.`);
          api.disconnect();
        } catch (e) {
          blockEvents = [];
        } finally {
          return { networkName: name, blockEvents };
        }
      });
      networkBlockEvents = await Promise.all(promises);
    });

    testIt("C100", `should not have UnsupportedVersion errors on DMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const dmpQueueEvents = events.filter(
            ({ event }) =>
              event.section.toString() === "dmpQueue" &&
              event.method.toString() === "UnsupportedVersion"
          );
          return { blockNum, dmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

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
        `Unexpected UnsupportedVersion XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C200", `should not have BadVersion errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events.filter(
            ({ event }) =>
              event.section.toString() === "xcmpQueue" && event.method.toString() === "BadVersion"
          );
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(`XCM error xcmpQueue.BadVersion in network ${networkName} block #${blockNum}.`)
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected BadVersion XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C300", `should not have Barrier errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "Barrier");
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(`Barrier XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`)
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected Barrier XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C400", `should not have Overflow errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "Overflow");
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(`Overflow XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`)
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected Overflow XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C500", `should not have MultiLocationFull errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "MultiLocationFull"
            );
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            "MultiLocationFull XCM error xcmpQueue. Fail in network " +
              networkName +
              " block #" +
              blockNum
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected MultiLocationFull XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C600", `should not have AssetNotFound errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "AssetNotFound");
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            `AssetNotFound XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected AssetNotFound XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C700", `should not have DestinationUnsupported errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "DestinationUnsupported"
            );
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            "DestinationUnsupported XCM error xcmpQueue.Fail in network " +
              networkName +
              " block #" +
              blockNum
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected DestinationUnsupported XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C800", `should not have Transport errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "Transport");
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(`Transport XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`)
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected Transport XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C900", `should not have FailedToDecode errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(({ event: { data } }) => (data as any).error.toString() === "FailedToDecode");
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            `FailedToDecode XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected FailedToDecode XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C1000", `should not have UnhandledXcmVersion errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "UnhandledXcmVersion"
            );
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            "UnhandledXcmVersion XCM error xcmpQueue.Fail in network " +
              networkName +
              " block #" +
              blockNum
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected UnhandledXcmVersion XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });

    testIt("C1100", `should not have WeightNotComputable errors on XCMP queue`, function () {
      const blockEvents = networkBlockEvents.map(({ networkName, blockEvents }) => {
        const filteredEvents = blockEvents.map(({ blockNum, events }) => {
          const xcmpQueueEvents = events
            .filter(
              ({ event }) =>
                event.section.toString() === "xcmpQueue" && event.method.toString() === "Fail"
            )
            .filter(
              ({ event: { data } }) => (data as any).error.toString() === "WeightNotComputable"
            );
          return { blockNum, xcmpQueueEvents };
        });
        return { networkName, errorEvents: filteredEvents };
      });

      const failures = blockEvents
        .map(({ networkName, errorEvents }) => {
          const filtered = errorEvents.filter((a) => a.xcmpQueueEvents.length !== 0);
          return { networkName, filtered };
        })
        .filter((a) => a.filtered.length > 0);

      failures.forEach(({ filtered, networkName }) =>
        filtered.forEach(({ blockNum }) =>
          debug(
            "WeightNotComputable XCM error xcmpQueue.Fail in network " +
              networkName +
              " block #" +
              blockNum
          )
        )
      );

      expect(
        failures.flatMap((a) => a).length,
        `Unexpected WeightNotComputable XCM errors in networks ${failures
          .map((a) => a.networkName)
          .join(`, `)}; please investigate.`
      ).to.equal(0);
    });
  }
);
