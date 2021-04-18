import anyTest, { TestInterface } from "ava";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { Contract } from "web3-eth-contract";
import { BlockHash } from "@polkadot/types/interfaces/chain";
import { ethers } from "ethers";
import { contractSources } from "../contracts/sources";
import { deployContractByName } from "./contracts";
import { startMoonbeamDevNode } from "./node";
import { provideWeb3Api, provideEthersApi, providePolkadotApi, EnhancedWeb3 } from "./providers";
import { ChildProcess } from "child_process";
import { createAndFinalizeBlock } from "./block";

export interface TestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApi: () => Promise<ApiPromise>;

  createAndFinalizeBlock: (
    parentHash?: BlockHash,
    finalize?: boolean
  ) => Promise<{
    duration: number;
    hash: BlockHash;
  }>;

  createBlockWith: <T>(
    promises: () => Promise<T>
  ) => Promise<{
    result: T;
    block: {
      duration: number;
      hash: BlockHash;
    };
  }>;

  deployContract: (name: string) => Promise<Contract>;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApi: ApiPromise;

  moonbeamProcess: ChildProcess;
}
export type MoonbeamDevTest = TestInterface<TestContext>;

// This is the variable to use inside test to have full access to the context
export const test = anyTest as MoonbeamDevTest;

interface InternalTestContext {
  polkadotWsProviders: WsProvider[];

  // Internal member to keep track of web3 singleton
  _polkadotApi: EnhancedWeb3;
}
type MoonbeamDevInternalTest = TestInterface<InternalTestContext & TestContext>;

const mbTest = anyTest as MoonbeamDevInternalTest;

mbTest.beforeEach("Setup: Start node", async (testInstance) => {
  const init = await startMoonbeamDevNode();
  testInstance.context.polkadotWsProviders = [];
  testInstance.context.moonbeamProcess = init.runningNode;

  testInstance.context.createWeb3 = async (protocol: "ws" | "http" = "http") =>
    protocol == "ws" ? provideWeb3Api(init.wsPort, "ws") : provideWeb3Api(init.rpcPort, "http");
  testInstance.context.createEthers = async () => provideEthersApi(init.rpcPort);
  testInstance.context.createPolkadotApi = async () => {
    const { provider, apiPromise } = await providePolkadotApi(init.wsPort);
    // We keep track of the polkadotWsProvider to close them at the end of the test
    if (!testInstance.context.polkadotWsProviders) {
      testInstance.context.polkadotWsProviders = [];
    }
    testInstance.context.polkadotWsProviders.push(provider);
    return apiPromise;
  };

  testInstance.context.web3 = await testInstance.context.createWeb3();
  testInstance.context.ethers = await testInstance.context.createEthers();
  testInstance.context.polkadotApi = await testInstance.context.createPolkadotApi();

  testInstance.context.createAndFinalizeBlock = (
    parentHash?: BlockHash,
    finalize: boolean = true
  ) => createAndFinalizeBlock(testInstance.context.polkadotApi, parentHash, finalize);

  testInstance.context.createBlockWith = async <T>(withBlock: () => Promise<T>) => {
    const promise = withBlock();
    const blockPromise = createAndFinalizeBlock(testInstance.context.polkadotApi);
    const result = await Promise.all([promise, blockPromise]);
    return {
      result: result[0],
      block: result[1],
    };
  };

  testInstance.context.deployContract = (name: string) =>
    deployContractByName(testInstance.context.web3, name);
});

mbTest.afterEach("Setup: Stop node", async (testInstance) => {
  if (testInstance.context.polkadotWsProviders) {
    testInstance.context.polkadotWsProviders.forEach((p) => p.disconnect());
  }
  testInstance.context.moonbeamProcess.kill();
  testInstance.context.moonbeamProcess = null;
});
