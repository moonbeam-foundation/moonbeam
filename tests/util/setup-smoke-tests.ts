import { ApiPromise } from "@polkadot/api";
import { providers } from "ethers";
import { SubstrateApi, EthersApi, ApiType } from "./wsApis";

const debug = require("debug")("test:setup");

export interface SmokeTestContext {
  polkadotApi: ApiPromise;
  relayApi?: ApiPromise;
  ethers: providers.WebSocketProvider;
}

// export type SmokeTestOptions = {
//   wssUrl: string;
//   relayWssUrl: string;
// };

export function describeSmokeSuite(
  title: string,
  // options: SmokeTestOptions,
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

      [context.polkadotApi, context.relayApi] = await Promise.all([
        SubstrateApi.api(ApiType.ParaChain),
        SubstrateApi.api(ApiType.RelayChain),
      ]);
      context.ethers = EthersApi.api();

      debug(`APIs retrieved for ${this.currentTest.title}`);
    });

    cb(context);
  });
}
