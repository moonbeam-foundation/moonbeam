import { ApiPromise } from "@polkadot/api";
import { MockProvider } from "@polkadot/rpc-provider/mock";
import { TypeRegistry } from "@polkadot/types";
import { providers } from "ethers";
import { SubstrateApi, EthersApi, ApiType } from "./wsApis";

const debug = require("debug")("test:setup");

export interface SmokeTestContext {
  polkadotApi: ApiPromise;
  relayApi: ApiPromise;
  ethers: providers.WebSocketProvider;
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
        await SubstrateApi.api(ApiType.ParaChain, options.wssUrl),
        options.relayWssUrl
          ? await SubstrateApi.api(ApiType.RelayChain, options.relayWssUrl)
          : unimplementedApi(),
        EthersApi.api(options.wssUrl),
      ]);

      await Promise.all([context.polkadotApi.isReady, context.relayApi.isReady]);

      debug(`Setup ready [${options.wssUrl}] for ${this.currentTest.title}`);
    });

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
