import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { NetworkTestArtifact, tracingTxns } from "../util/tracing-txns";
import Bottleneck from "bottleneck";
import { providers } from "ethers";

const debug = require("debug")("smoke:historic-compatibility");
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
const httpEndpoint = process.env.HTTP_URL;

describeSmokeSuite(`Verifying historic compatibility...`, async (context) => {
  let traceStatic: NetworkTestArtifact;
  before("Loading tracing static data", async function () {
    const chainId = (await context.polkadotApi.query.ethereumChainId.chainId()).toString();
    debug(`Running tracing tests against chainId ${chainId}.`);
    traceStatic = tracingTxns.find((a) => a.chainId.toString() == chainId);

    if (!traceStatic) {
      debug(`No test data available for ChainId ${chainId}, skipping test.`);
      this.skip();
    }
  });

  it("can debugTrace for all previous runtimes", async function () {
    if (httpEndpoint == null) {
      debug(`No HTTP_URL provided, skipping test.`);
      this.skip();
    }
    this.timeout(300000);
    const provider = new providers.JsonRpcProvider(httpEndpoint);
    const promises = traceStatic.testData.map(async (a) => {
      try {
        const result = await limiter.schedule(() =>
          provider.send("debug_traceTransaction", [a.txHash])
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
        debug(
          `Failure tracing in runtime ${a.runtime}, blocknumber ${a.blockNumber} ` + `: ${a.result}`
        );
        return true;
      }
    });
    expect(failures).to.be.empty;
  });

  it("can get receipt for historic transactions", async function () {
    this.timeout(300000);
    const promises = traceStatic.testData.map(async (a) => {
      try {
        const result = await limiter.schedule(() =>
          context.ethers.send("eth_getTransactionReceipt", [a.txHash])
        );
        debug(`Successful response from runtime ${a.runtime} in block #${a.blockNumber}.`);
        const error = result == null;
        return { runtime: a.runtime, blockNumber: a.blockNumber, error, result };
      } catch (e) {
        return { runtime: a.runtime, blockNumber: a.blockNumber, error: true, result: e };
      }
    });

    const results = await Promise.all(promises.flatMap((a) => a));
    const failures = results.filter((a) => {
      if (a.error === true) {
        debug(
          `Failure fetching txn receipt on runtime ${a.runtime}, blocknumber ${a.blockNumber}` +
            ` and result: ${JSON.stringify(a.result)}`
        );
        return true;
      }
    });
    expect(failures).to.be.empty;
  });
});
