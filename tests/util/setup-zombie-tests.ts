import "@polkadot/api-augment";

import { ApiPromise } from "@polkadot/api";
import chalk from "chalk";
import { ethers } from "ethers";

import zombie from "@parity/zombienet";
import { HttpProvider } from "web3-core";
import { EnhancedWeb3, provideEthersApi, provideWeb3Api } from "./providers";
import { Network } from "@parity/zombienet/dist/network";

const debug = require("debug")("test:setup");

const ZOMBIENET_CREDENTIALS = process.env.ZOMBIENET_CREDENTIALS || "./zombienet/moonbase.env";
const ZOMBIENET_CONFIG = process.env.ZOMBIENET_CONFIG || "../zombienet/default-config.json";

let network: Network;
export interface ZombieTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApiParachain: (parachainNumber: number) => Promise<ApiPromise>;
  createPolkadotApiParachains: () => Promise<ApiPromise>;
  createPolkadotApiRelaychains: () => Promise<ApiPromise>;
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

export interface InternalZombieTestContext extends ZombieTestContext {
  _web3Providers: HttpProvider[];
}

export function describeZombienet(title: string, cb: (context: InternalZombieTestContext) => void) {
  describe(title, function () {
    // Increases timeout for tests as they rely on real block production
    this.timeout(32000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalZombieTestContext = {} as InternalZombieTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test ZombieNet", async function () {
      this.timeout(120000);
      try {
        const networkConfig = require(ZOMBIENET_CONFIG);
        network = await zombie.start(ZOMBIENET_CREDENTIALS, networkConfig, {
          spawnConcurrency: 10,
        });
        // Context is given prior to this assignement, so doing
        // context = init.context will fail because it replace the variable;

        context.network = network;
        context.blockNumber = 0;
        context._web3Providers = [];

        context.createWeb3 = async (protocol: "ws" | "http" = "http") => {
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
