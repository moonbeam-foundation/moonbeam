import { ApiPromise, WsProvider } from "@polkadot/api";
import { ApiOptions } from "@polkadot/api/types";
import { types } from "moonbeam-types-bundle";
import { setTimeout } from "timers/promises";
import { providers } from "ethers";

let relayApi: SubstrateApi;
let polkadotApi: SubstrateApi;
let ethApi: EthersApi;

export enum ApiType {
  ParaChain,
  RelayChain,
  Ethers,
}

export class SubstrateApi {
  private _api?: ApiPromise;
  private _provider?: WsProvider;

  async connect(wssUrl: string, options: ApiOptions) {
    if (!wssUrl) {
      this._api = null;
    } else {
      this._provider = new WsProvider(wssUrl, 100);

      this._provider.on("error", async (error) => {
        console.error(error);
        console.log("Pausing before reconnecting..");
        await setTimeout(100);
      });

      this._api = await ApiPromise.create({
        provider: this._provider,
        noInitWarn: true,
        ...options,
      });

      // Necessary hack to allow polkadotApi to finish its internal metadata loading
      // apiPromise.isReady unfortunately doesn't wait for those properly
      await setTimeout(100);
      await this._api?.isReadyOrError;
    }

    return this;
  }

  disconnect() {
    if (this._api) {
      this._api.disconnect();
      delete this._api;
    }
  }

  public static async api(networkType: ApiType, wssUrl?: string) {
    switch (networkType) {
      case ApiType.RelayChain:
        if (!relayApi) {
          relayApi = await new SubstrateApi().connect(wssUrl, {
            initWasm: false,
          });
        }
        return relayApi._api;
      case ApiType.ParaChain:
        if (!polkadotApi) {
          polkadotApi = await new SubstrateApi().connect(wssUrl, {
            initWasm: false,
            typesBundle: types,
          });
        }
        return polkadotApi._api;
      case ApiType.Ethers:
        throw new Error("Wrong API Type");
    }
  }

  public static async close(networkType: ApiType) {
    switch (networkType) {
      case ApiType.RelayChain:
        if (relayApi) {
          relayApi.disconnect();
        }
        break;
      case ApiType.ParaChain:
        if (polkadotApi) {
          polkadotApi.disconnect();
        }
        break;
      case ApiType.Ethers:
        throw new Error("Wrong API type");
    }
  }
}

export class EthersApi {
  private _api?: providers.WebSocketProvider;

  connect(wssUrl: string) {
    this._api = new providers.WebSocketProvider(wssUrl);
    return this;
  }

  disconnect() {
    this._api.removeAllListeners();
    this._api._websocket.terminate();
    delete this._api;
  }

  public static api(wssUrl?: string) {
    if (!ethApi) {
      ethApi = new EthersApi().connect(wssUrl);
    }
    return ethApi._api;
  }

  public static async close() {
    if (ethApi) {
      ethApi.disconnect();
    }
  }
}
