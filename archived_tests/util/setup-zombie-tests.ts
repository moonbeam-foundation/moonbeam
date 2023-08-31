import "@polkadot/api-augment";

import { ApiPromise } from "@polkadot/api";
import chalk from "chalk";
import { ethers } from "ethers";
import { ILoader, Environment } from "nunjucks";

import fs from "fs";

import zombie from "@zombienet/orchestrator";
import { HttpProvider } from "web3-core";
import { EnhancedWeb3, provideEthersApi, provideWeb3Api } from "./providers";
import { Network } from "@zombienet/orchestrator";
import {
  getMoonbeamDockerBinary,
  getMoonbeamReleaseBinary,
  getPlainSpecsFromTag,
  getPolkadotReleaseBinary,
} from "./binaries";
import { ParaTestOptions } from "./para-node";
import { BINARY_PATH, RELAY_BINARY_PATH } from "./constants";
import { UpgradePreferences, upgradeRuntime } from "./upgrade";

const debug = require("debug")("test:setup");

const ZOMBIENET_CREDENTIALS = process.env.ZOMBIENET_CREDENTIALS || "./zombienet/moonbase.env";

let network: Network;
export interface ZombieTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApiParachain: (parachainNumber: number) => Promise<ApiPromise>;
  createPolkadotApiParachains: () => Promise<ApiPromise>;
  createPolkadotApiRelaychains: () => Promise<ApiPromise>;
  upgradeRuntime: (preferences: UpgradePreferences) => Promise<number>;
  waitBlocks: (count: number) => Promise<number>; // return current block when the promise resolves
  blockNumber: number;
  network: Network;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApiParaone: ApiPromise;
}

export interface ParachainApis {
  parachainId: number;
  apis: ApiPromise[];
}

export class RelativeLoader implements ILoader {
  constructor(private paths: string[]) {}
  getSource(name: string) {
    const fullPath = require.resolve(name, {
      paths: this.paths,
    });

    return {
      src: fs.readFileSync(fullPath, "utf-8"),
      path: fullPath,
      noCache: true,
    };
  }
}

export interface InternalZombieTestContext extends ZombieTestContext {
  _web3Providers: HttpProvider[];
}
export interface ZombieTestConfig {
  runtime: "moonbase" | "moonriver" | "moonbeam";
}

export function describeZombienet(
  title: string,
  options: ParaTestOptions,
  cb: (context: InternalZombieTestContext) => void
) {
  describe(title, function () {
    // Increases timeout for tests as they rely on real block production
    this.timeout(32000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalZombieTestContext = {} as InternalZombieTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test ZombieNet", async function () {
      this.timeout(320000);
      try {
        // Retrieve the correct binary/specs

        const paraBinary =
          !options.parachain.binary || options.parachain.binary == "local"
            ? BINARY_PATH
            : await getMoonbeamReleaseBinary(options.parachain.binary);
        console.log(options.parachain);
        const paraSpecs =
          "spec" in options.parachain
            ? options.parachain.spec
            : !("runtime" in options.parachain) || options.parachain.runtime == "local"
            ? options.parachain.chain || "moonbase-local"
            : await getPlainSpecsFromTag(
                options.parachain.chain || "moonbase-local",
                options.parachain.runtime
              );
        const chainSpecCommand =
          "runtime" in options.parachain
            ? await getMoonbeamDockerBinary(options.parachain.runtime)
            : paraBinary;

        const relayBinary =
          !options?.relaychain?.binary || options?.relaychain?.binary == "local"
            ? RELAY_BINARY_PATH
            : await getPolkadotReleaseBinary(options.relaychain.binary);

        const networkFile = "zombienet/default-config.json";

        const env = new Environment();
        const templateContent = fs.readFileSync(networkFile).toString();
        const content = env.renderString(templateContent, {
          ...process.env,
          binaryPath: paraBinary,
          relayBinaryPath: relayBinary,
          chain: paraSpecs,
          runtime: "moonbase",
          // see https://github.com/paritytech/zombienet/issues/467
          chainSpecCommand, // not yet working :(
        });
        console.log(content);
        const networkConfig = JSON.parse(content);

        network = await zombie.start(ZOMBIENET_CREDENTIALS, networkConfig, {
          spawnConcurrency: 10,
        });
        // Context is given prior to this assignement, so doing
        // context = init.context will fail because it replace the variable;

        context.network = network;
        context.blockNumber = 0;
        context._web3Providers = [];

        context.createWeb3 = async (protocol: "ws" | "http" = "ws") => {
          if (protocol == "http") {
            throw new Error("http protocol is not yet supported by zombienet");
          }
          const provider = await provideWeb3Api(Object.values(network.paras)[0].nodes[0].wsUri);
          context._web3Providers.push((provider as any)._provider);
          return provider;
        };
        context.createEthers = async () =>
          provideEthersApi(Object.values(network.paras)[0].nodes[0].wsUri);
        context.createPolkadotApiParachain = async (parachainNumber: number) => {
          const parachainId = Object.keys(network.paras)[0];
          if (!network.paras[parachainId].nodes[0].apiInstance) {
            await network.paras[parachainId].nodes[0].connectApi();
          }
          return network.paras[parachainId].nodes[0].apiInstance as any;
        };
        context.createPolkadotApiParachains = async () => {
          const apiPromises = await Promise.all(
            Object.values(network.paras).map(async (parachain) => {
              if (!parachain.nodes[0].apiInstance) {
                await parachain.nodes[0].connectApi();
              }
              return parachain.nodes[0].apiInstance.isReady;
            })
          );
          await new Promise((resolve) => {
            setTimeout(resolve, 100);
          });

          return apiPromises[0] as any;
        };
        context.upgradeRuntime = (preferences) =>
          upgradeRuntime(context.polkadotApiParaone, preferences);
        context.createPolkadotApiRelaychains = async () => {
          const apiPromises = await Promise.all(
            Object.values(network.relay).map(async (relay) => {
              if (!relay.apiInstance) {
                await relay.connectApi();
              }
              await relay.apiInstance.isReady;
              return relay.apiInstance;
            })
          );
          await new Promise((resolve) => {
            setTimeout(resolve, 100);
          });

          return apiPromises[0] as any;
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

        context.waitBlocks = async (count: number) => {
          return new Promise<number>((resolve) => {
            pendingCallbacks.push({
              blockNumber: (context.blockNumber || 0) + count,
              resolve,
            });
          });
        };

        context.polkadotApiParaone = await context.createPolkadotApiParachains();
        subBlocks(context.polkadotApiParaone);

        context.web3 = await context.createWeb3();
        context.ethers = await context.createEthers();
        await context.waitBlocks(1);
        debug(
          `Setup ready [${/:([0-9]+)$/.exec((context.web3.currentProvider as any).url)}] for ${
            this.currentTest.title
          }`
        );
        network.showNetworkInfo(networkConfig.settings.provider);
      } catch (e) {
        console.error(`Failed to start nodes !!!`);
        console.error(e);
        process.exit(1);
      }
    });

    after(async function () {
      this.timeout(10000);
      debug(`Cleaning`);
      await Promise.all(context._web3Providers.map((p) => p.disconnect()));
      if (network) {
        await network.stop();
        debug(`Network stopped`);
        network = null;
      }
    });

    cb(context);
  });
}
