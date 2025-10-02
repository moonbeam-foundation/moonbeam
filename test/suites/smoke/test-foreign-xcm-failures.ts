import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { getBlockArray, TEN_MINS } from "@moonwall/util";
import type { FrameSystemEventRecord } from "@polkadot/types/lookup";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { rateLimiter, checkTimeSliceForUpgrades } from "../../helpers/common.js";
import { ForeignChainsEndpoints, getEndpoints } from "../../helpers/foreign-chains.js";

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : TEN_MINS;
const limiter = rateLimiter();

type BlockEventsRecord = {
  blockNum: number;
  events: FrameSystemEventRecord[];
};

type NetworkBlockEvents = {
  networkName: string;
  blockEvents: BlockEventsRecord[];
};

let skip = false;

describeSuite({
  id: "S13",
  title:
    `Foreign XCM Failures in past ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours` +
    ` should not be serious`,
  foundationMethods: "read_only",
  notChainType: "moonbase",
  testCases: ({ context, it, log }) => {
    const networkBlockEvents: NetworkBlockEvents[] = [];
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const networkName = paraApi.runtimeChain.toString();
      const foreignChainInfos = ForeignChainsEndpoints.find(
        (a) => a.moonbeamNetworkName === networkName
      );

      if (foreignChainInfos == null) {
        log(`No Foreign chain endpoints available for network ${networkName}, skipping.`);
        skip = true;
        return; // TODO: replace with skip() when added to vitest
      }

      const relayName =
        networkName === "Moonbeam"
          ? "Polkadot"
          : networkName === "Moonriver"
            ? "Kusama"
            : "Unsupported";
      let chainsWithRpcs = foreignChainInfos.foreignChains.map((chain) => {
        const endpoints = getEndpoints(relayName, chain.paraId);
        return { ...chain, endpoints };
      });

      chainsWithRpcs = chainsWithRpcs.map((chain) => {
        return {
          ...chain,
          endpoints: chain.endpoints.filter(
            (value) => value.startsWith("ws://") || value.startsWith("wss://")
          ),
        };
      });

      for (const { name, endpoints, mutedUntil = 0 } of chainsWithRpcs) {
        let blockEvents: BlockEventsRecord[] = [];

        if (!endpoints.length) {
          console.warn(`Parachain ${name} did not provide any public endpoints`);
          continue;
        }

        if (mutedUntil && mutedUntil >= new Date().getTime()) {
          log(`Network tests for ${name} has been muted, skipping.`);
          return { networkName: name, blockEvents: [] };
        }

        let api: ApiPromise;
        try {
          log(`Connecting to ${name}...`);
          console.debug(`Endpoints: `, endpoints);
          for (const endpoint of endpoints) {
            const provider = new WsProvider(endpoint);
            await provider.connectWithRetry();
            api = new ApiPromise({ provider });
            // INFO: api.isReady can get stuck, while api.isReadyOrError does not
            if (await api.isReadyOrError.then(() => true).catch(() => false)) {
              break;
            }
          }
          // Make sure the connection is ready
          await api.isReadyOrError;
          console.debug(`Connected to ${name}...`);

          const blockNumArray = await getBlockArray(api, timePeriod);

          // Determine if the block range intersects with an upgrade event
          const { result, specVersion: onChainRt } = await checkTimeSliceForUpgrades(
            api,
            blockNumArray,
            api.consts.system.version.specVersion
          );
          if (result) {
            log(
              `Time slice of blocks intersects with upgrade from RT ${onChainRt}, skipping chain.`
            );
            continue;
          }

          const getEvents = async (blockNum: number) => {
            const blockHash = await limiter.schedule(() => api.rpc.chain.getBlockHash(blockNum));
            const apiAt = await limiter.schedule(() => api.at(blockHash));
            const events = await limiter.schedule(() => apiAt.query.system.events());
            return { blockNum, events };
          };

          blockEvents = await Promise.all(blockNumArray.map((num) => getEvents(num)));
          log(`Finished loading blocks for ${name}.`);
        } catch (e) {
          expect.fail(`Could not connect to parachain: ${name}`);
        } finally {
          await api.disconnect();
          networkBlockEvents.push({ networkName: name, blockEvents });
        }
      }
    }, TEN_MINS);

    it({
      id: "C100",
      title: "should not have UnsupportedVersion errors on DMP queue",
      test: function () {
        if (skip) {
          return;
        }

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
            log(
              `XCM error dmpQueue.UnsupportedVersion in network ${networkName} block #${blockNum}.`
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected UnsupportedVersion XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C200",
      title: "should not have BadVersion errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(`XCM error xcmpQueue.BadVersion in network ${networkName} block #${blockNum}.`)
          )
        );

        expect(
          failures.flat().length,
          `Unexpected BadVersion XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C300",
      title: "should not have Barrier errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(`Barrier XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`)
          )
        );

        expect(
          failures.flat().length,
          `Unexpected Barrier XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C400",
      title: "should not have Overflow errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(`Overflow XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`)
          )
        );

        expect(
          failures.flat().length,
          `Unexpected Overflow XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C500",
      title: "should not have MultiLocationFull errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(
              "MultiLocationFull XCM error xcmpQueue. Fail in network " +
                networkName +
                " block #" +
                blockNum
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected MultiLocationFull XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C600",
      title: "should not have AssetNotFound errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(
              `AssetNotFound XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected AssetNotFound XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C700",
      title: "should not have DestinationUnsupported errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(
              "DestinationUnsupported XCM error xcmpQueue.Fail in network " +
                networkName +
                " block #" +
                blockNum
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected DestinationUnsupported XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C800",
      title: "should not have Transport errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(`Transport XCM error xcmpQueue.Fail in network ${networkName} block #${blockNum}.`)
          )
        );

        expect(
          failures.flat().length,
          `Unexpected Transport XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C900",
      title: "should not have FailedToDecode errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(
              "FailedToDecode XCM error xcmpQueue." +
                `Fail in network ${networkName} block #${blockNum}.`
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected FailedToDecode XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1000",
      title: "should not have UnhandledXcmVersion errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(
              "UnhandledXcmVersion XCM error xcmpQueue.Fail in network " +
                networkName +
                " block #" +
                blockNum
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected UnhandledXcmVersion XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });

    it({
      id: "C1100",
      title: "should not have WeightNotComputable errors on XCMP queue",
      test: function () {
        if (skip) {
          return;
        }
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
            log(
              "WeightNotComputable XCM error xcmpQueue.Fail in network " +
                networkName +
                " block #" +
                blockNum
            )
          )
        );

        expect(
          failures.flat().length,
          `Unexpected WeightNotComputable XCM errors in networks ${failures
            .map((a) => a.networkName)
            .join(`, `)}; please investigate.`
        ).to.equal(0);
      },
    });
  },
});
