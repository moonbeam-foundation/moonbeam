import "@polkadot/api-augment";

import { ApiPromise } from "@polkadot/api";
import chalk from "chalk";
import { ethers } from "ethers";
import child_process from "child_process";
import { HttpProvider } from "web3-core";

import { DEBUG_MODE } from "./constants";
import {
  NodePorts,
  ParachainPorts,
  ParaTestOptions,
  startParachainNodes,
  stopParachainNodes,
} from "./para-node";
import { EnhancedWeb3, provideEthersApi, providePolkadotApi, provideWeb3Api } from "./providers";
import { UpgradePreferences, upgradeRuntime } from "./upgrade";

const debug = require("debug")("test:setup");

const PORT_PREFIX = (process.env.PORT_PREFIX && parseInt(process.env.PORT_PREFIX)) || -1;

export interface ParaTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApiParachain: (parachainNumber: number) => Promise<ApiPromise>;
  createPolkadotApiParachains: () => Promise<ApiPromise>;
  createPolkadotApiRelaychains: () => Promise<ApiPromise>;
  waitBlocks: (count: number) => Promise<number>; // return current block when the promise resolves
  upgradeRuntime: (preferences: UpgradePreferences) => Promise<number>;
  blockNumber: number;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApiParaone: ApiPromise;
}

export interface ParachainApis {
  parachainId: number;
  apis: ApiPromise[];
}

export interface InternalParaTestContext extends ParaTestContext {
  _polkadotApiParachains: ParachainApis[];
  _polkadotApiRelaychains: ApiPromise[];
  _web3Providers: HttpProvider[];
}

export function describeParachain(
  title: string,
  options: ParaTestOptions,
  cb: (context: InternalParaTestContext) => void
) {
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout("spec" in options.parachain ? 3600000 : 300000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalParaTestContext = {} as InternalParaTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      try {
        const init = !DEBUG_MODE
          ? await startParachainNodes(options)
          : PORT_PREFIX == -1
          ? {
              paraPorts: [
                {
                  parachainId: 1000,
                  ports: [
                    {
                      p2pPort: 30333,
                      rpcPort: 9944,
                    },
                  ],
                },
              ],
              relayPorts: [],
            }
          : {
              paraPorts: [
                {
                  parachainId: 1000,
                  ports: [
                    {
                      p2pPort: PORT_PREFIX * 1000 + 100,
                      rpcPort: PORT_PREFIX * 1000 + 101,
                    },
                  ],
                },
              ],
              relayPorts: [],
            };
        // Context is given prior to this assignement, so doing
        // context = init.context will fail because it replace the variable;

        context._polkadotApiParachains = [];
        context._polkadotApiRelaychains = [];
        context._web3Providers = [];
        context.blockNumber = 0;

        context.createWeb3 = async (protocol: "ws" | "http" = "http") => {
          const provider =
            protocol == "ws"
              ? await provideWeb3Api(`ws://localhost:${init.paraPorts[0].ports[0].rpcPort}`)
              : await provideWeb3Api(`http://localhost:${init.paraPorts[0].ports[0].rpcPort}`);
          context._web3Providers.push((provider as any)._provider);
          return provider;
        };
        context.createEthers = async () =>
          provideEthersApi(`http://localhost:${init.paraPorts[0].ports[0].rpcPort}`);
        context.createPolkadotApiParachain = async (parachainNumber: number) => {
          const promise = providePolkadotApi(init.paraPorts[parachainNumber].ports[0].rpcPort);
          context._polkadotApiParachains.push({
            parachainId: init.paraPorts[parachainNumber].parachainId,
            apis: [await promise],
          });
          return promise;
        };
        context.createPolkadotApiParachains = async () => {
          const apiPromises = await Promise.all(
            init.paraPorts.map(async (parachain: ParachainPorts) => {
              return {
                parachainId: parachain.parachainId,
                apis: await Promise.all(
                  parachain.ports.map(async (ports: NodePorts) => {
                    return providePolkadotApi(ports.rpcPort);
                  })
                ),
              };
            })
          );
          // We keep track of the polkadotApis to close them at the end of the test
          context._polkadotApiParachains = apiPromises;
          await Promise.all(
            apiPromises.map(async (promises) =>
              Promise.all(promises.apis.map((promise) => promise.isReady))
            )
          );
          // Necessary hack to allow polkadotApi to finish its internal metadata loading
          // apiPromise.isReady unfortunately doesn't wait for those properly
          await new Promise((resolve) => {
            setTimeout(resolve, 100);
          });

          return apiPromises[0].apis[0];
        };
        context.createPolkadotApiRelaychains = async () => {
          const apiPromises = await Promise.all(
            init.relayPorts.map(async (ports: NodePorts) => {
              return await providePolkadotApi(ports.rpcPort, true);
            })
          );
          // We keep track of the polkadotApis to close them at the end of the test
          context._polkadotApiRelaychains = apiPromises;
          await Promise.all(apiPromises.map((promise) => promise.isReady));
          // Necessary hack to allow polkadotApi to finish its internal metadata loading
          // apiPromise.isReady unfortunately doesn't wait for those properly
          await new Promise((resolve) => {
            setTimeout(resolve, 100);
          });

          return apiPromises[0];
        };

        let pendingCallbacks: {
          blockNumber: number;
          resolve: (blockNumber: number) => void;
        }[] = [];
        const subBlocks = async (api: ApiPromise) => {
          return api.rpc.chain.subscribeNewHeads(async (header) => {
            context.blockNumber = header.number.toNumber();
            if (context.blockNumber == 0) {
              console.log(
                `Start listening for new blocks. Production will start in ${chalk.red(`1 minute`)}`
              );
            }
            debug(`New block: #${context.blockNumber}`);

            let i = pendingCallbacks.length;
            while (i--) {
              const pendingCallback = pendingCallbacks[i];
              if (pendingCallback.blockNumber <= context.blockNumber) {
                pendingCallbacks.splice(i, 1);
                pendingCallback.resolve(context.blockNumber);
              }
            }
          });
        };

        context.polkadotApiParaone = await context.createPolkadotApiParachains();
        subBlocks(context.polkadotApiParaone);

        context.waitBlocks = async (count: number) => {
          return new Promise<number>((resolve) => {
            pendingCallbacks.push({
              blockNumber: (context.blockNumber || 0) + count,
              resolve,
            });
          });
        };

        context.upgradeRuntime = (preferences) =>
          upgradeRuntime(context.polkadotApiParaone, preferences);
        context.web3 = await context.createWeb3();
        context.ethers = await context.createEthers();
        debug(
          `Setup ready [${/:([0-9]+)$/.exec((context.web3.currentProvider as any).host)[1]}] for ${
            this.currentTest.title
          }`
        );
      } catch (e) {
        console.error(`Failed to start nodes !!!`);
        console.error(e);
        process.exit(1);
      }
    });

    after(async function () {
      await Promise.all(context._web3Providers.map((p) => p.disconnect()));
      await Promise.all(
        context._polkadotApiParachains.map(
          async (ps) => await Promise.all(ps.apis.map((p) => p.disconnect()))
        )
      );
      await Promise.all(context._polkadotApiRelaychains.map((p) => p.disconnect()));

      if (!DEBUG_MODE) {
        await stopParachainNodes();
        await new Promise((resolve) => {
          // TODO: Replace Sleep by actually checking the process has ended
          setTimeout(resolve, 1000);
        });
      }
    });

    cb(context);
  });
}

export interface RuntimeUpgradeVersions {
  // Version of Moonbase runtime as written in the local source code
  localVersion: string;
  // latest version released on Github
  latestReleasedVersion: string;
  // Previous version of Moonbase runtime needed to execute the runtime upgrade
  previousVersion: string;
  // Does the current source code contain authoring changes
  hasAuthoringChanges: boolean;
}

function runOrDefault(cmd: string, def = ""): string {
  try {
    return child_process.execSync(cmd).toString().trim();
  } catch (e) {
    return def;
  }
}

export function retrieveParaVersions() {
  const localVersion = runOrDefault(
    `grep 'spec_version: [0-9]*' ../runtime/moonbase/src/lib.rs | grep -o '[0-9]*'`
  );
  const localAuthoringVersion = runOrDefault(
    `grep 'authoring_version: [0-9]*' ../runtime/moonbase/src/lib.rs | grep -o '[0-9]*'`
  );

  const isAlreadyReleased =
    runOrDefault(
      `git tag -l -n 'runtime-[0-9]*' | cut -d' ' -f 1 | cut -d'-' -f 2 | grep "${localVersion}"`
    ) == localVersion;

  const previousVersion = runOrDefault(
    `git tag -l -n 'runtime-[0-9]*' | cut -d' ' -f 1 | cut -d'-' -f 2 ` +
      `| sed '1 i ${localVersion}' | sort -n -r ` +
      `| uniq | grep -A1 "${localVersion}" | tail -1`
  );
  const previousAuthoringVersion = runOrDefault(
    `git show runtime-${previousVersion}:../runtime/moonbase/src/lib.rs ` +
      `| grep 'authoring_version: [0-9]*' | grep -o '[0-9]*'`
  );

  // List authoring_version from git commit since the previous runtime being used
  // and find if there is a new version.
  const authoringChanges = runOrDefault(
    `git grep authoring_version ` +
      `$(git rev-list runtime-${previousVersion}..HEAD -- ../runtime/moonbase/src/lib.rs) ` +
      `-- ../runtime/moonbase/src/lib.rs ` +
      `| grep -v "$(git grep authoring_version runtime-${previousVersion} ` +
      `-- ../runtime/moonbase/src/lib.rs ` +
      `| grep -o 'authoring_version:\ *[0-9]')" ` +
      `| grep -o 'authoring_version:\ *[0-9]*' || exit 0`
  );

  const hasAuthoringChanges =
    !!authoringChanges || previousAuthoringVersion != localAuthoringVersion;

  debug(
    `Using previous runtime ${previousVersion} ` +
      `(authoring changes: ${hasAuthoringChanges} - ` +
      `localVersion: ${localVersion} - release: ${isAlreadyReleased})`
  );

  return {
    localVersion,
    previousVersion,
    hasAuthoringChanges,
  };
}
