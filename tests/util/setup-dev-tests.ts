import { ApiPromise } from "@polkadot/api";
import { JsonRpcResponse } from "web3-core-helpers";
import type { BlockHash } from "@polkadot/types/interfaces/chain/types";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { cryptoWaitReady } from "@polkadot/util-crypto";
import { RegistryError } from "@polkadot/types/types";
import { EventRecord } from "@polkadot/types/interfaces";

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
import { SPAWNING_TIME, DEBUG_MODE } from "./constants";
import { HttpProvider } from "web3-core";
import { extractError, ExtrinsicCreation } from "./substrate-rpc";
import { alith } from "./accounts";

const debug = require("debug")("test:setup");

export interface BlockCreation {
  parentHash?: BlockHash;
  finalize?: boolean;
}
export interface SubBlockCreation<
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
> extends BlockCreation {
  transactions: Call[];
}

export interface BlockCreationResponse {
  block: {
    duration: number;
    hash: BlockHash;
  };
}
export interface EthBlockCreationResponse<T extends string | string[]>
  extends BlockCreationResponse {
  result: T extends string[] ? JsonRpcResponse[] : JsonRpcResponse;
}

export interface SubBlockCreationResponse<
  ApiType extends ApiTypes,
  Call extends SubmittableExtrinsic<ApiType> | SubmittableExtrinsic<ApiType>[]
> extends BlockCreationResponse {
  result: Call extends SubmittableExtrinsic<ApiType>[] ? ExtrinsicCreation[] : ExtrinsicCreation;
}

export interface DevTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApi: () => Promise<ApiPromise>;

  createBlockWithEth: <T extends string | string[]>(
    transactions: T,
    options?: BlockCreation
  ) => Promise<EthBlockCreationResponse<T>>;
  createBlock: (options?: BlockCreation) => Promise<BlockCreationResponse>;
  createBlockWithExtrinsic<
    Call extends SubmittableExtrinsic<ApiType> | SubmittableExtrinsic<ApiType>[],
    ApiType extends ApiTypes
  >(
    transactions: Call,
    options?: BlockCreation
  ): Promise<SubBlockCreationResponse<ApiType, Call>>;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApi: ApiPromise;
  rpcPort: number;
  ethTransactionType?: EthTransactionType;
}

interface InternalDevTestContext extends DevTestContext {
  _polkadotApis: ApiPromise[];
  _web3Providers: HttpProvider[];
}

type EthTransactionType = "Legacy" | "EIP2930" | "EIP1559";

export function describeDevMoonbeam(
  title: string,
  cb: (context: DevTestContext) => void,
  ethTransactionType: EthTransactionType = "Legacy",
  withWasm?: boolean
) {
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout(5000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalDevTestContext = { ethTransactionType } as InternalDevTestContext;

    // The currently running node for this describe
    let moonbeamProcess: ChildProcess;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      this.timeout(SPAWNING_TIME);
      const init = !DEBUG_MODE
        ? await startMoonbeamDevNode(withWasm)
        : {
            runningNode: null,
            p2pPort: 19931,
            wsPort: 19933,
            rpcPort: 19932,
          };
      moonbeamProcess = init.runningNode;
      context.rpcPort = init.rpcPort;

      // Context is given prior to this assignement, so doing
      // context = init.context will fail because it replace the variable;

      context._polkadotApis = [];
      context._web3Providers = [];
      moonbeamProcess = init.runningNode;

      context.createWeb3 = async (protocol: "ws" | "http" = "http") => {
        const provider =
          protocol == "ws"
            ? await provideWeb3Api(init.wsPort, "ws")
            : await provideWeb3Api(init.rpcPort, "http");
        context._web3Providers.push((provider as any)._provider);
        return provider;
      };
      context.createEthers = async () => provideEthersApi(init.rpcPort);
      context.createPolkadotApi = async () => {
        const apiPromise = await providePolkadotApi(init.wsPort);
        // We keep track of the polkadotApis to close them at the end of the test
        context._polkadotApis.push(apiPromise);
        await apiPromise.isReady;
        // Necessary hack to allow polkadotApi to finish its internal metadata loading
        // apiPromise.isReady unfortunately doesn't wait for those properly
        await new Promise((resolve) => {
          setTimeout(resolve, 100);
        });

        return apiPromise;
      };

      context.polkadotApi = await context.createPolkadotApi();
      context.web3 = await context.createWeb3();
      context.ethers = await context.createEthers();

      context.createBlock = async <T>(options: BlockCreation = {}) => {
        let { parentHash, finalize } = options;
        return {
          block: await createAndFinalizeBlock(context.polkadotApi, parentHash, finalize),
        };
      };

      context.createBlockWithEth = async (
        transactions: string | string[],
        options?: BlockCreation
      ): Promise<EthBlockCreationResponse<string | string[]>> => {
        if (Array.isArray(transactions)) {
          const result = await Promise.all(
            transactions.map((t) => customWeb3Request(context.web3, "eth_sendRawTransaction", [t]))
          );
          return {
            result,
            block: (await context.createBlock()).block,
          };
        }
        const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
          transactions,
        ]);
        const block = (await context.createBlock(options)).block;
        // Adds extra time to avoid empty transaction when querying it
        await Promise.resolve((resolve) => setTimeout(resolve, 10));
        return {
          result,
          block,
        };
      };

      context.createBlockWithExtrinsic = async <ApiType extends ApiTypes>(
        transactions: SubmittableExtrinsic<ApiType> | SubmittableExtrinsic<ApiType>[],
        options?: BlockCreation
      ): Promise<
        SubBlockCreationResponse<
          ApiType,
          SubmittableExtrinsic<ApiType> | SubmittableExtrinsic<ApiType>[]
        >
      > => {
        // This should return a  string, but is a bit complex to handle type
        // properly so any will suffice
        const extrinsicHashes: string[] = [];
        const txs = Array.isArray(transactions) ? transactions : [transactions];
        for (const call of txs) {
          if (call.isSigned) {
            extrinsicHashes.push((await call.send()).toString());
          } else {
            extrinsicHashes.push((await call.signAndSend(alith)).toString());
          }
        }

        const blockResult = await context.createBlock(options);

        // We retrieve the events for that block
        const allRecords: EventRecord[] = (await (
          await context.polkadotApi.at(blockResult.block.hash)
        ).query.system.events()) as any;

        // We retrieve the block (including the extrinsics)
        const blockData = await context.polkadotApi.rpc.chain.getBlock(blockResult.block.hash);

        const result: ExtrinsicCreation[] = extrinsicHashes.map((extrinsicHash) => {
          const extrinsicIndex = blockData.block.extrinsics.findIndex(
            (ext) => ext.hash.toHex() == extrinsicHash
          );
          if (extrinsicIndex < 0) {
            throw new Error(
              `Extrinsic ${extrinsicHashes} is missing in the block ${blockResult.block.hash}`
            );
          }
          blockData.block.extrinsics[extrinsicIndex] as any;

          // We retrieve the events associated with the extrinsic
          const events = allRecords.filter(
            ({ phase }) =>
              phase.isApplyExtrinsic && phase.asApplyExtrinsic.toNumber() == extrinsicIndex
          );
          const failed = extractError(events);
          return {
            extrinsic: blockData.block.extrinsics[extrinsicIndex],
            events,
            error:
              failed &&
              ((failed.isModule && context.polkadotApi.registry.findMetaError(failed.asModule)) ||
                ({ name: failed.toString() } as RegistryError)),
            successful: !failed,
          };
        });

        return {
          block: blockResult.block,
          result: Array.isArray(transactions) ? result : result[0],
        };
      };

      debug(
        `Setup ready [${/:([0-9]+)$/.exec((context.web3.currentProvider as any).host)[1]}] for ${
          this.currentTest.title
        }`
      );
    });

    after(async function () {
      await Promise.all(context._web3Providers.map((p) => p.disconnect()));
      await Promise.all(context._polkadotApis.map((p) => p.disconnect()));

      if (moonbeamProcess) {
        await new Promise((resolve) => {
          moonbeamProcess.once("exit", resolve);
          moonbeamProcess.kill();
          moonbeamProcess = null;
        });
      }
    });

    cb(context);
  });
}

export function describeDevMoonbeamAllEthTxTypes(
  title: string,
  cb: (context: DevTestContext) => void,
  withWasm?: boolean
) {
  let wasm = withWasm !== undefined ? withWasm : false;
  describeDevMoonbeam(title + " (Legacy)", cb, "Legacy", wasm);
  describeDevMoonbeam(title + " (EIP1559)", cb, "EIP1559", wasm);
  describeDevMoonbeam(title + " (EIP2930)", cb, "EIP2930", wasm);
}
