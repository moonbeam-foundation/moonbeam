import { ApiPromise } from "@polkadot/api";
import { ethers } from "ethers";
import { provideWeb3Api, provideEthersApi, providePolkadotApi, EnhancedWeb3 } from "./providers";
import { DEBUG_MODE } from "./constants";
import { HttpProvider } from "web3-core";
import { NodePorts, ParachainOptions, startParachainNodes, stopParachainNodes } from "./para-node";
const debug = require("debug")("test:setup");

export interface ParaTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApiParachains: () => Promise<ApiPromise>;
  createPolkadotApiRelaychains: () => Promise<ApiPromise>;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApiParaone: ApiPromise;
}

export interface InternalParaTestContext extends ParaTestContext {
  _polkadotApiParachains: ApiPromise[][];
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
              [
                {
                  p2pPort: 19931,
                  wsPort: 19933,
                  rpcPort: 19932,
                },
              ],
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
            ? await provideWeb3Api(init.paraPorts[0][0].wsPort, "ws")
            : await provideWeb3Api(init.paraPorts[0][0].rpcPort, "http");
        context._web3Providers.push((provider as any)._provider);
        return provider;
      };
      context.createEthers = async () => provideEthersApi(init.paraPorts[0][0].rpcPort);
      context.createPolkadotApiParachains = async () => {
        const apiPromises = await Promise.all(
          init.paraPorts.map(async (parachain: NodePorts[]) => {
            return Promise.all(
              parachain.map(async (ports: NodePorts) => {
                return await providePolkadotApi(ports.wsPort);
              })
            );
          })
        );
        // We keep track of the polkadotApis to close them at the end of the test
        context._polkadotApiParachains = apiPromises;
        await Promise.all(
          apiPromises.map(async (promises) =>
            Promise.all(promises.map((promise) => promise.isReady))
          )
        );
        // Necessary hack to allow polkadotApi to finish its internal metadata loading
        // apiPromise.isReady unfortunately doesn't wait for those properly
        await new Promise((resolve) => {
          setTimeout(resolve, 100);
        });

        return apiPromises[0][0];
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

      context.polkadotApiParaone = await context.createPolkadotApiParachains();
      await context.createPolkadotApiRelaychains();
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
          async (ps) => await Promise.all(ps.map((p) => p.disconnect()))
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
