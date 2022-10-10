import { ApiPromise } from "@polkadot/api";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { EventRecord } from "@polkadot/types/interfaces";
import { RegistryError } from "@polkadot/types/types";
import { ChildProcess } from "child_process";
import { ethers } from "ethers";
import { HttpProvider } from "web3-core";

import { alith } from "./accounts";
import { createAndFinalizeBlock } from "./block";
import { DEBUG_MODE, SPAWNING_TIME } from "./constants";
import { RuntimeChain, startMoonbeamDevNode } from "./dev-node";
import {
  customWeb3Request,
  EnhancedWeb3,
  provideEthersApi,
  providePolkadotApi,
  provideWeb3Api,
} from "./providers";
import { extractError, ExtrinsicCreation } from "./substrate-rpc";

import type { BlockHash } from "@polkadot/types/interfaces/chain/types";
const debug = require("debug")("test:setup");

export interface BlockCreation {
  parentHash?: string;
  finalize?: boolean;
}

export interface BlockCreationResponse<
  ApiType extends ApiTypes,
  Call extends SubmittableExtrinsic<ApiType> | string | (SubmittableExtrinsic<ApiType> | string)[]
> {
  block: {
    duration: number;
    hash: string;
  };
  result: Call extends (string | SubmittableExtrinsic<ApiType>)[]
    ? ExtrinsicCreation[]
    : ExtrinsicCreation;
}

export interface DevTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApi: () => Promise<ApiPromise>;

  createBlock<
    ApiType extends ApiTypes,
    Call extends
      | SubmittableExtrinsic<ApiType>
      | Promise<SubmittableExtrinsic<ApiType>>
      | string
      | Promise<string>,
    Calls extends Call | Call[]
  >(
    transactions?: Calls,
    options?: BlockCreation
  ): Promise<
    BlockCreationResponse<ApiType, Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>>
  >;

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
  runtime: RuntimeChain = "moonbase",
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
        ? await startMoonbeamDevNode(withWasm, runtime)
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
            ? await provideWeb3Api(`ws://localhost:${init.wsPort}`)
            : await provideWeb3Api(`http://localhost:${init.rpcPort}`);
        context._web3Providers.push((provider as any)._provider);
        return provider;
      };
      context.createEthers = async () => provideEthersApi(`http://localhost:${init.rpcPort}`);
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

      context.createBlock = async <
        ApiType extends ApiTypes,
        Call extends
          | SubmittableExtrinsic<ApiType>
          | Promise<SubmittableExtrinsic<ApiType>>
          | string
          | Promise<string>,
        Calls extends Call | Call[]
      >(
        transactions?: Calls,
        options: BlockCreation = {}
      ) => {
        const results: ({ type: "eth"; hash: string } | { type: "sub"; hash: string })[] = [];
        const txs =
          transactions == undefined
            ? []
            : Array.isArray(transactions)
            ? transactions
            : [transactions];
        for await (const call of txs) {
          if (typeof call == "string") {
            // Ethereum
            results.push({
              type: "eth",
              hash: (await customWeb3Request(context.web3, "eth_sendRawTransaction", [call]))
                .result,
            });
          } else if (call.isSigned) {
            results.push({
              type: "sub",
              hash: (await call.send()).toString(),
            });
          } else {
            results.push({
              type: "sub",
              hash: (await call.signAndSend(alith)).toString(),
            });
          }
        }

        const { parentHash, finalize } = options;
        const blockResult = await createAndFinalizeBlock(context.polkadotApi, parentHash, finalize);

        // No need to extract events if no transactions
        if (results.length == 0) {
          return {
            block: blockResult,
            result: null,
          };
        }

        // We retrieve the events for that block
        const allRecords: EventRecord[] = (await (
          await context.polkadotApi.at(blockResult.hash)
        ).query.system.events()) as any;
        // We retrieve the block (including the extrinsics)
        const blockData = await context.polkadotApi.rpc.chain.getBlock(blockResult.hash);

        const result: ExtrinsicCreation[] = results.map((result) => {
          const extrinsicIndex =
            result.type == "eth"
              ? allRecords
                  .find(
                    ({ phase, event: { section, method, data } }) =>
                      phase.isApplyExtrinsic &&
                      section == "ethereum" &&
                      method == "Executed" &&
                      data[2].toString() == result.hash
                  )
                  ?.phase?.asApplyExtrinsic?.toNumber()
              : blockData.block.extrinsics.findIndex((ext) => ext.hash.toHex() == result.hash);
          // We retrieve the events associated with the extrinsic
          const events = allRecords.filter(
            ({ phase }) =>
              phase.isApplyExtrinsic && phase.asApplyExtrinsic.toNumber() === extrinsicIndex
          );
          const failure = extractError(events);
          return {
            extrinsic: extrinsicIndex >= 0 ? blockData.block.extrinsics[extrinsicIndex] : null,
            events,
            error:
              failure &&
              ((failure.isModule && context.polkadotApi.registry.findMetaError(failure.asModule)) ||
                ({ name: failure.toString() } as RegistryError)),
            successful: extrinsicIndex !== undefined && !failure,
            hash: result.hash,
          };
        });

        // Adds extra time to avoid empty transaction when querying it
        if (results.find((r) => r.type == "eth")) {
          await new Promise((resolve) => setTimeout(resolve, 2));
        }
        return {
          block: blockResult,
          result: Array.isArray(transactions) ? result : (result[0] as any),
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
  describeDevMoonbeam(title + " (Legacy)", cb, "Legacy", "moonbase", wasm);
  describeDevMoonbeam(title + " (EIP1559)", cb, "EIP1559", "moonbase", wasm);
  describeDevMoonbeam(title + " (EIP2930)", cb, "EIP2930", "moonbase", wasm);
}
