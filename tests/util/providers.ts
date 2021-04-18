import Web3 from "web3";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { typesBundle } from "../../moonbeam-types-bundle";
import { JsonRpcResponse } from "web3-core-helpers";
import { ethers } from "ethers";
import { GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

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
            `Failed to send custom request (${method} (${params.join(",")})): ${
              error.message || error.toString()
            }`
          );
        }
        resolve(result);
      }
    );
  });
}

export type EnhancedWeb3 = Web3 & {
  customRequest: (method: string, params: any[]) => Promise<JsonRpcResponse>;
};

export const provideWeb3Api = async (port: number, protocol: "ws" | "http" = "http") => {
  const web3 =
    protocol == "ws"
      ? new Web3(`ws://localhost:${port}`) // TODO: restore support for
      : new Web3(`http://localhost:${port}`);

  // Adding genesis account for convenience
  web3.eth.accounts.wallet.add(GENESIS_ACCOUNT_PRIVATE_KEY);

  // Hack to add customRequest method.
  (web3 as any).customRequest = (method: string, params: any[]) =>
    customWeb3Request(web3, method, params);

  return web3 as EnhancedWeb3;
};

export const providePolkadotApi = async (port: number) => {
  const provider = new WsProvider(`ws://localhost:${port}`);
  return {
    provider,
    apiPromise: await ApiPromise.create({
      provider: provider,
      typesBundle: typesBundle as any,
    }),
  };
};

export const provideEthersApi = async (port: number) => {
  return new ethers.providers.JsonRpcProvider(`http://localhost:${port}`);
};
