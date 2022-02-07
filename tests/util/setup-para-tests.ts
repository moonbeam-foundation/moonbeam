import "@polkadot/api-augment";
import { ApiPromise } from "@polkadot/api";
import { ethers } from "ethers";
import { provideWeb3Api, provideEthersApi, providePolkadotApi, EnhancedWeb3 } from "./providers";
import { DEBUG_MODE } from "./constants";
import { HttpProvider } from "web3-core";
import fs from "fs";
import chalk from "chalk";
import {
  getRuntimeWasm,
  NodePorts,
  ParachainOptions,
  ParachainPorts,
  startParachainNodes,
  stopParachainNodes,
} from "./para-node";
import { KeyringPair } from "@substrate/txwrapper-core";
const debug = require("debug")("test:setup");

export interface ParaTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApiParachain: (parachainNumber) => Promise<ApiPromise>;
  createPolkadotApiParachains: () => Promise<ApiPromise>;
  createPolkadotApiRelaychains: () => Promise<ApiPromise>;
  waitBlocks: (count: number) => Promise<number>; // return current block when the promise resolves
  upgradeRuntime: (
    from: KeyringPair,
    runtimeName: "moonbase" | "moonriver" | "moonbeam",
    runtimeVersion: string
  ) => Promise<void>;
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
  options: ParachainOptions,
  cb: (context: InternalParaTestContext) => void
) {
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout(300000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalParaTestContext = {} as InternalParaTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      this.timeout(300000);
      const init = !DEBUG_MODE
        ? await startParachainNodes(options)
        : {
            paraPorts: [
              {
                parachainId: 1000,
                ports: [
                  {
                    p2pPort: 19931,
                    wsPort: 19933,
                    rpcPort: 19932,
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

      context.createWeb3 = async (protocol: "ws" | "http" = "http") => {
        const provider =
          protocol == "ws"
            ? await provideWeb3Api(init.paraPorts[0].ports[0].wsPort, "ws")
            : await provideWeb3Api(init.paraPorts[0].ports[0].rpcPort, "http");
        context._web3Providers.push((provider as any)._provider);
        return provider;
      };
      context.createEthers = async () => provideEthersApi(init.paraPorts[0].ports[0].rpcPort);
      context.createPolkadotApiParachain = async (parachainNumber: number) => {
        const promise = providePolkadotApi(init.paraPorts[parachainNumber].ports[0].wsPort);
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
                  return providePolkadotApi(ports.wsPort);
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
            return await providePolkadotApi(ports.wsPort, true);
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

      let pendingPromises = [];
      const subBlocks = async (api) => {
        return api.rpc.chain.subscribeNewHeads(async (header) => {
          context.blockNumber = header.number.toNumber();
          if (context.blockNumber == 0) {
            console.log(
              `Start listening for new blocks. Production will start in ${chalk.red(`1 minute`)}`
            );
          }

          let i = pendingPromises.length;
          while (i--) {
            const pendingPromise = pendingPromises[i];
            if (pendingPromise.blockNumber <= context.blockNumber) {
              pendingPromises.splice(i, 1);
              pendingPromise.resolve(context.blockNumber);
            }
          }
        });
      };

      context.polkadotApiParaone = await context.createPolkadotApiParachains();
      subBlocks(context.polkadotApiParaone);

      context.waitBlocks = async (count: number) => {
        return new Promise<number>((resolve) => {
          pendingPromises.push({
            blockNumber: context.blockNumber + count,
            resolve,
          });
        });
      };

      context.upgradeRuntime = async (
        from: KeyringPair,
        runtimeName: "moonbase" | "moonriver" | "moonbeam",
        runtimeVersion: string
      ) => {
        return new Promise<void>(async (resolve) => {
          const code = fs
            .readFileSync(await getRuntimeWasm(runtimeName, runtimeVersion))
            .toString();

          process.stdout.write(
            `Sending sudo.setCode (${code.slice(0, 6)}...${code.slice(-6)} [~${Math.floor(
              code.length / 1024
            )} kb])...`
          );
          await context.polkadotApiParaone.tx.sudo
            .sudoUncheckedWeight(
              await context.polkadotApiParaone.tx.system.setCodeWithoutChecks(code),
              1
            )
            .signAndSend(from);
          process.stdout.write(`✅\n`);

          process.stdout.write(`Waiting to apply new runtime (${chalk.red(`~4min`)})...`);
          let isInitialVersion = true;
          const unsub = await context.polkadotApiParaone.rpc.state.subscribeRuntimeVersion(
            async (version) => {
              if (!isInitialVersion) {
                console.log(
                  `✅ New runtime ${version.implName.toString()} ${version.specVersion.toString()}`
                );
                unsub();
                await context.waitBlocks(1); // Wait for next block to have the new runtime applied
                resolve();
              }
              isInitialVersion = false;
            }
          );
          process.stdout.write(`✅\n`);
        });
      };
      context.web3 = await context.createWeb3();
      context.ethers = await context.createEthers();
      debug(
        `Setup ready [${/:([0-9]+)$/.exec((context.web3.currentProvider as any).host)[1]}] for ${
          this.currentTest.title
        }`
      );
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
