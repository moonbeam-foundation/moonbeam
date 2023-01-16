import { ApiPromise } from "@polkadot/api";
import { providers } from "ethers";
import { SubstrateApi, EthersApi, ApiType } from "./wsApis";

const debug = require("debug")("test:setup");

export interface SmokeTestContext {
  polkadotApi: ApiPromise;
  relayApi?: ApiPromise;
  ethers: providers.WebSocketProvider;
}

export function describeSmokeSuite(
  suiteNumber: string,
  title: string,
  cb: (context: SmokeTestContext) => void
) {
  describe(`${title} #${suiteNumber}`, function () {
    this.timeout(23700);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: SmokeTestContext = {} as SmokeTestContext;

    before("Starting Moonbeam Smoke Suite", async function () {
      this.timeout(10000);

      [context.polkadotApi, context.relayApi] = await Promise.all([
        SubstrateApi.api(ApiType.ParaChain),
        SubstrateApi.api(ApiType.RelayChain),
      ]);
      context.ethers = EthersApi.api();

      debug(`APIs retrieved for ${this.currentTest.title}`);
    });

    beforeEach(function () {
      this.currentTest.title += `::${suiteNumber}`;
    });

    cb(context);
  });
}
