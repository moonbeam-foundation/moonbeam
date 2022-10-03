import { ApiPromise, WsProvider } from "@polkadot/api";
import { ethers } from "ethers";
import Web3 from "web3";
import { Log } from "web3-core";
import { JsonRpcResponse } from "web3-core-helpers";
import { Subscription as Web3Subscription } from "web3-core-subscriptions";
import { BlockHeader } from "web3-eth";

import { typesBundlePre900 } from "moonbeam-types-bundle";
import { alith, ALITH_PRIVATE_KEY } from "./accounts";
import { MIN_GAS_PRICE } from "./constants";

export async function customWeb3Request(web3: Web3, method: string, params: any[]) {
  return new Promise<JsonRpcResponse>((resolve, reject) => {
    (web3.currentProvider as any).send(
      {
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
      },
      (error: Error | null, result?: JsonRpcResponse) => {
        if (error) {
          reject(
            `Failed to send custom request (${method} (${params
              .map((p) => {
                const str = p.toString();
                return str.length > 128 ? `${str.slice(0, 96)}...${str.slice(-28)}` : str;
              })
              .join(",")})): ${error.message || error.toString()}`
          );
        }
        resolve(result);
      }
    );
  });
}

export interface Web3EthCallOptions {
  from?: string | number;
  to: string;
  value?: number | string | bigint;
  gas?: number | string;
  gasPrice?: number | string | bigint;
  maxPriorityFeePerGas?: number | string | bigint;
  maxFeePerGas?: number | string | bigint;
  data?: string;
  nonce?: number;
}

export async function web3EthCall(web3: Web3, options: Web3EthCallOptions) {
  return await customWeb3Request(web3, "eth_call", [
    {
      from: options.from == undefined ? options.from : alith.address,
      value: options.value,
      gas: options.gas == undefined ? options.gas : 256000,
      gasPrice: options.gas == undefined ? options.gas : `0x${MIN_GAS_PRICE}`,
      to: options.to,
      data: options.data,
    },
  ]);
}

// Extra type because web3 is not well typed
export interface Subscription<T> extends Web3Subscription<T> {
  once: (type: "data" | "connected", handler: (data: T) => void) => Subscription<T>;
}

// Little helper to hack web3 that are not complete.
export function web3Subscribe(web3: Web3, type: "newBlockHeaders"): Subscription<BlockHeader>;
export function web3Subscribe(web3: Web3, type: "pendingTransactions"): Subscription<string>;
export function web3Subscribe(web3: Web3, type: "logs", params: {}): Subscription<Log>;
export function web3Subscribe(
  web3: Web3,
  type: "newBlockHeaders" | "pendingTransactions" | "logs",
  params?: any
) {
  return (web3.eth as any).subscribe(...[].slice.call(arguments, 1));
}

export type EnhancedWeb3 = Web3 & {
  customRequest: (method: string, params: any[]) => Promise<JsonRpcResponse>;
};

export const provideWeb3Api = async (uri: string) => {
  const web3 = new Web3(uri);

  // Adding genesis account for convenience
  web3.eth.accounts.wallet.add(ALITH_PRIVATE_KEY);

  // Hack to add customRequest method.
  (web3 as any).customRequest = (method: string, params: any[]) =>
    customWeb3Request(web3, method, params);

  return web3 as EnhancedWeb3;
};

export const providePolkadotApi = async (port: number, isNotMoonbeam?: boolean) => {
  return isNotMoonbeam
    ? await ApiPromise.create({
        initWasm: false,
        provider: new WsProvider(`ws://localhost:${port}`),
      })
    : await ApiPromise.create({
        provider: new WsProvider(`ws://localhost:${port}`),
        typesBundle: typesBundlePre900 as any,
      });
};

export const provideEthersApi = async (uri: string) => {
  return new ethers.providers.JsonRpcProvider(uri);
};
