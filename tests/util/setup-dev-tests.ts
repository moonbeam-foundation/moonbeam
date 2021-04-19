import { ApiPromise, WsProvider } from "@polkadot/api";
import { JsonRpcResponse } from "web3-core-helpers";
import { BlockHash } from "@polkadot/types/interfaces/chain";
import { ethers } from "ethers";
import { startMoonbeamDevNode } from "./dev-node";
import {
  provideWeb3Api,
  provideEthersApi,
  providePolkadotApi,
  EnhancedWeb3,
  customWeb3Request,
} from "./providers";
import { ChildProcess } from "child_process";
import { createAndFinalizeBlock } from "./block";
import { SPAWNING_TIME } from "./constants";

export interface BlockCreation {
  parentHash?: BlockHash;
  finalize?: boolean;
  transactions?: string[];
}

export interface DevTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApi: () => Promise<ApiPromise>;

  createBlock: (
    options?: BlockCreation
  ) => Promise<{
    txResults: JsonRpcResponse[];
    block: {
      duration: number;
      hash: BlockHash;
    };
  }>;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApi: ApiPromise;

  moonbeamProcess: ChildProcess;
}

interface InternalDevTestContext extends DevTestContext {
  polkadotWsProviders: WsProvider[];

  // Internal member to keep track of web3 singleton
  _polkadotApi: EnhancedWeb3;
}

export function describeDevMoonbeam(title: string, cb: (context: DevTestContext) => void) {
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout(5000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalDevTestContext = {} as InternalDevTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      this.timeout(SPAWNING_TIME);
      const init = await startMoonbeamDevNode();
      // Context is given prior to this assignement, so doing
      // context = init.context will fail because it replace the variable;

      context.polkadotWsProviders = [];
      context.moonbeamProcess = init.runningNode;

      context.createWeb3 = async (protocol: "ws" | "http" = "http") =>
        protocol == "ws" ? provideWeb3Api(init.wsPort, "ws") : provideWeb3Api(init.rpcPort, "http");
      context.createEthers = async () => provideEthersApi(init.rpcPort);
      context.createPolkadotApi = async () => {
        const { provider, apiPromise } = await providePolkadotApi(init.wsPort);
        // We keep track of the polkadotWsProvider to close them at the end of the test
        if (!context.polkadotWsProviders) {
          context.polkadotWsProviders = [];
        }
        context.polkadotWsProviders.push(provider);
        await apiPromise.isReady;
        return apiPromise;
      };

      context.web3 = await context.createWeb3();
      context.ethers = await context.createEthers();
      context.polkadotApi = await context.createPolkadotApi();

      context.createBlock = async <T>(options: BlockCreation = {}) => {
        let { parentHash, finalize, transactions = [] } = options;

        let txResults = await Promise.all(
          transactions.map((t) => customWeb3Request(context.web3, "eth_sendRawTransaction", [t]))
        );
        const block = await createAndFinalizeBlock(context.polkadotApi, parentHash, finalize);
        return {
          txResults,
          block,
        };
      };
    });

    after(async function () {
      // console.log(`\x1b[31m Killing RPC\x1b[0m`);
      if (context.polkadotWsProviders) {
        await Promise.all(context.polkadotWsProviders.map((p) => p.disconnect()));
      }
      context.moonbeamProcess.kill();
      context.moonbeamProcess = null;
    });

    cb(context);
  });
}
