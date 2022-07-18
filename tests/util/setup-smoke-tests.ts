import { ApiPromise, WsProvider } from "@polkadot/api";

const debug = require("debug")("test:setup");

export interface SmokeTestContext {
  // We also provided singleton providers for simplicity
  polkadotApi: ApiPromise;
  relayApi: ApiPromise;
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
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout(23700);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: SmokeTestContext = {} as SmokeTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Smoke Suite", async function () {
      this.timeout(10000);

      if (!options.wssUrl) {
        throw Error(`Missing wssUrl parameter (use WSS_URL=... npm run smoke-test)`);
      }

      [context.polkadotApi, context.relayApi] = await Promise.all([
        ApiPromise.create({
          initWasm: false,
          provider: new WsProvider(options.wssUrl),
        }),
        ApiPromise.create({
          initWasm: false,
          provider: new WsProvider(options.relayWssUrl),
        }),
      ]);

      await Promise.all([context.polkadotApi.isReady, context.relayApi.isReady]);
      // Necessary hack to allow polkadotApi to finish its internal metadata loading
      // apiPromise.isReady unfortunately doesn't wait for those properly
      await new Promise((resolve) => {
        setTimeout(resolve, 100);
      });

      debug(`Setup ready [${options.wssUrl}] for ${this.currentTest.title}`);
    });

    after(async function () {
      await context.polkadotApi.disconnect();
      await context.relayApi.disconnect();
    });

    cb(context);
  });
}
