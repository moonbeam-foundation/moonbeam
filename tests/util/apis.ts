import { ApiPromise, WsProvider } from "@polkadot/api";
import { ApiOptions } from "@polkadot/api/types";
import { types } from "moonbeam-types-bundle";
import { ethers } from "ethers";

export class SubstrateApi {
  private _api?: ApiPromise;

  public async init(options: ApiOptions) {
    await this.connect(options);

    // Necessary hack to allow polkadotApi to finish its internal metadata loading
    // apiPromise.isReady unfortunately doesn't wait for those properly
    await new Promise((resolve) => {
      setTimeout(resolve, 100);
    });
    await this._api?.isReadyOrError;

    return this;
  }

  private async connect(options: ApiOptions) {
    this._api = await ApiPromise.create(options);
    this._api.on("error", async (e) => {
      console.log(`Api error: ${JSON.stringify(e)}, reconnecting....`);
      await this.connect(options);
    });
  }

  public async disconnect() {
    await this._api.disconnect();
  }

  public get api() {
    return this._api;
  }
}

export class EthersApi {
  private _api?;

  public init(wssUrl: string) {
    this.connect(wssUrl);
    return this;
  }

  public connect(wssUrl: string) {
    this._api = new ethers.providers.WebSocketProvider(wssUrl);
  }

  public async disconnect() {
    await this._api.destroy();
  }

  public get api() {
    return this._api;
  }
}

let relayApi: SubstrateApi;
let polkadotApi: SubstrateApi;
let ethApi: EthersApi;

export async function closeApi(networkType: "relay" | "parachain" | "ethers") {
  switch (networkType) {
    case "relay":
      if (relayApi) {
        await relayApi.disconnect();
      }
      break;
    case "parachain":
      if (polkadotApi) {
        await polkadotApi.disconnect();
      }
      break;
    case "ethers":
      if (ethApi) {
        await ethApi.disconnect();
      }
      break;
  }
}

export async function getApi(networkType: "relay" | "parachain" | "ethers", wssUrl: string) {
  switch (networkType) {
    case "relay":
      if (!relayApi) {
        relayApi = await new SubstrateApi().init({
          initWasm: false,
          provider: new WsProvider(wssUrl),
        });
      }
      return relayApi.api;
    case "parachain":
      if (!polkadotApi) {
        polkadotApi = await new SubstrateApi().init({
          initWasm: false,
          provider: new WsProvider(wssUrl),
          typesBundle: types,
        });
      }
      return polkadotApi.api;
    case "ethers":
      if (!ethApi) {
        ethApi = new EthersApi().init(wssUrl);
      }
      return ethApi.api;
  }
}
