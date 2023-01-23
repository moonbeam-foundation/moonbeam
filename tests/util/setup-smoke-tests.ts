import { ApiPromise } from "@polkadot/api";
import { providers } from "ethers";
import { SubstrateApi, EthersApi, ApiType } from "./wsApis";

export interface SmokeTestContext {
  polkadotApi: ApiPromise;
  relayApi?: ApiPromise;
  ethers: providers.WebSocketProvider;
}

interface CustomTest {
  (id: string, title: string, cb: () => void, only?: boolean): void;
}

export function describeSmokeSuite(
  suiteNumber: string,
  title: string,
  cb: (context: SmokeTestContext, testIt: CustomTest) => void
) {
  describe(`🗃️  #${suiteNumber} ${title}`, function () {
    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: SmokeTestContext = {} as SmokeTestContext;

    function testIt(id: string, title: string, cb: () => void, only = false) {
      !only
        ? it(`📁  #${suiteNumber.concat(id)} ${title}`, cb)
        : it.only(`📁  #${suiteNumber.concat(id)} ${title}`, cb);
    }

    before("Starting Moonbeam Smoke Suite", async function () {
      this.timeout(10000);

      [context.polkadotApi, context.relayApi] = await Promise.all([
        SubstrateApi.api(ApiType.ParaChain),
        SubstrateApi.api(ApiType.RelayChain),
      ]);
      context.ethers = EthersApi.api();
    });

    cb(context, testIt);
  });
}
