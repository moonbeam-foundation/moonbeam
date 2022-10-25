import { ApiPromise, WsProvider } from "@polkadot/api";
import { MockProvider } from "@polkadot/rpc-provider/mock";
import { TypeRegistry } from "@polkadot/types";
import { types } from "moonbeam-types-bundle";
import { ethers } from "ethers";
import { getApi } from "./apis";

const debug = require("debug")("test:setup");

export interface SmokeTestContext {
  // We also provided singleton providers for simplicity
  polkadotApi: ApiPromise;
  relayApi: ApiPromise;
  ethers: ethers.providers.WebSocketProvider;
}

export type SmokeTestOptions = {
  wssUrl: string;
  relayWssUrl: string;
};

export function describeSmokeSuite(
  title: string,
  options: SmokeTestOptions,
  cb: (context: SmokeTestContext) => void
) {
  if (!options.wssUrl) {
    throw Error(`Missing wssUrl parameter (use WSS_URL=... npm run smoke-test)`);
  }

  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout(23700);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: SmokeTestContext = {} as SmokeTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Smoke Suite", async function () {
      this.timeout(10000);

      [context.polkadotApi, context.relayApi, context.ethers] = await Promise.all([
        await getApi("parachain", options.wssUrl),
        options.relayWssUrl ? await getApi("relay", options.relayWssUrl) : unimplementedApi(),
        await getApi("ethers", options.wssUrl),
      ]);

      await Promise.all([context.polkadotApi.isReady, context.relayApi.isReady]);

      debug(`Setup ready [${options.wssUrl}] for ${this.currentTest.title}`);
    });

    // after(async function () {});

    cb(context);
  });
}

async function unimplementedApi() {
  return new Proxy(
    await ApiPromise.create({
      initWasm: false,
      provider: new MockProvider(new TypeRegistry()),
    }),
    {
      get(target, prop, receiver) {
        switch (prop) {
          case "isReady":
            return Promise.resolve(true);
          case "tx":
          case "query":
          case "consts":
            throw new Error("unimplemented! Requires `RELAY_WSS_URL` parameter");
        }
        return Reflect.get(target, prop, receiver);
      },
    }
  );
}
