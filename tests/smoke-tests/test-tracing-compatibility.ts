import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { NetworkTestArtifact, tracingTxns } from "../util/tracing-txns";
import Bottleneck from "bottleneck";
import { providers } from "ethers";

const debug = require("debug")("smoke:tracing-compatibility");
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
const httpEndpoint = process.env.HTTP_URL;

describeSmokeSuite(`Verifying tracing compatibility...`, async (context) => {
  let traceStatic: NetworkTestArtifact;
  before("Loading tracing static data", async function () {
    const chainId = (await context.polkadotApi.query.ethereumChainId.chainId()).toString();
    debug(`Running tracing tests against chainId ${chainId}.`);
    traceStatic = tracingTxns.find((a) => a.chainId.toString() == chainId);

    if (!traceStatic) {
      debug(`No test data available for ChainId ${chainId}, skipping test.`);
      this.skip();
    }

    if (httpEndpoint == null) {
      debug(`No HTTP_URL provided, skipping test.`);
      this.skip();
    }
  });

  //
  it("can debugTrace for all previous runtimes", async function () {
    this.timeout(300000);
    const provider = new providers.JsonRpcProvider(httpEndpoint);

    const promises = traceStatic.testData.map(async (a) => {
      try {
        const result = await limiter.schedule(() =>
          provider.send("debug_traceTransaction", [
            a.txHash,
            { disableStorage: true, disableMemory: true },
          ])
        );
        debug(`Successful tracing response from runtime ${a.runtime} in block #${a.blockNumber}.`);
        return { runtime: a.runtime, blockNumber: a.blockNumber, error: false, result };
      } catch (e) {
        return { runtime: a.runtime, blockNumber: a.blockNumber, error: true, result: e };
      }
    });

    const results = await Promise.all(promises.flatMap((a) => a));
    const failures = results.filter((a) => {
      if (a.error === true) {
        debug(`Failure tracing in runtime ${a.runtime} blocknumber ${a.blockNumber} with `);
        return true;
      }
    });
    expect(failures).to.be.empty;
  });
});
