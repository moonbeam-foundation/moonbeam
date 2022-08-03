import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { hexToBigInt } from "@polkadot/util";
import chalk from "chalk";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

// TEMPLATE: Remove useless types at the end
import type { PalletProxyProxyDefinition } from "@polkadot/types/lookup";
import { InferencePriority } from "typescript";

// TEMPLATE: Replace debug name
const debug = require("debug")("smoke:randomness");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify number of proxies per account`, { wssUrl, relayWssUrl }, (context) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;

  const requests: { id: number, state: any}[] = [];
  let numRequests: number = 0; // our own count
  let requestCount: number = 0; // from pallet storage

  before("Retrieve all requests", async function () {
    // It takes time to load all the proxies.
    // TEMPLATE: Adapt the timeout to be an over-estimate
    this.timeout(30_000); // 30s

    const limit = 1000;
    let last_key = "";
    let count = 0;

    atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );

    // TEMPLATE: query the data
    while (true) {
      let query = await apiAt.query.randomness.requests.entriesPaged({
        args: [],
        pageSize: limit,
        startKey: last_key,
      });

      if (query.length == 0) {
        break;
      }
      count += query.length;

      // TEMPLATE: convert the data into the format you want (usually a dictionary per account)
      for (const request of query) {
        const key = request[0].toHex();
        expect(key.length >= 18, "storage key should be at least 64 bits"); // assumes "0x"

        const requestIdEncoded = key.slice(-16);
        const requestId = hexToBigInt(requestIdEncoded, { isLe: true });

        requests.push({id: Number(requestId), state: request[1]});
        numRequests += 1;
        last_key = key;
      }

      // Debug logs to make sure it keeps progressing
      // TEMPLATE: Adapt log line
      if (true || count % (10 * limit) == 0) {
        debug(`Retrieved ${count} requests`);
        debug(`Requests: ${requests}`);
      }
    }

    requestCount = ((await apiAt.query.randomness.requestCount()) as any).toNumber();

    // TEMPLATE: Adapt proxies
    debug(`Retrieved ${count} total proxies`);
  });

  it("should have fewer Requests than RequestCount", async function () {
    this.timeout(10000);

    const numOutstandingRequests = numRequests;
    expect(numOutstandingRequests).to.be.lessThanOrEqual(requestCount);
  });

  it("should not have requestId above RequestCount", async function () {
    this.timeout(1000);

    const highestId = requests.reduce((prev, request) => Math.max(request.id, prev), 0);
    expect(highestId).to.be.lessThanOrEqual(requestCount);
  });

  it("should not have results without a matching request", async function () {
    this.timeout(10000);

    let query = await apiAt.query.randomness.randomnessResults.entries();
    await query.forEach(([key, results]) => {
      // offset is:
      // * 2 for "0x"
      // * 32 for module
      // * 32 for method
      // * 16 for the hashed part of the key: the twox64(someRequestType) part
      // the remaining substr after offset is the concat part, which we can decode with createType
      const offset = 2 + 32 + 32 + 16;
      const requestTypeEncoded = key.toHex().slice(offset);
      const requestType = context.polkadotApi.registry.createType(
        `PalletRandomnessRequestType`,
        "0x" + requestTypeEncoded
      );

      // sanity check
      expect(
        (requestType as any).isBabeEpoch || (requestType as any).isLocal,
        "unexpected enum in encoded RequestType string"
      );

      if ((requestType as any).isBabeEpoch) {
        let epoch = (requestType as any).asBabeEpoch;
        // TODO
      } else {
        // look for any requests which depend on the "local" block
        let block = (requestType as any).asLocal;
        let found = requests.find((request) => {
          // TODO: can we traverse this hierarchy of types without creating each?
          const requestState = context.polkadotApi.registry.createType(
            "PalletRandomnessRequestState",
            request.state.toHex()
          );
          const requestRequest = context.polkadotApi.registry.createType(
            "PalletRandomnessRequest",
            (requestState as any).request.toHex(),
          );
          const requestInfo = context.polkadotApi.registry.createType(
            "PalletRandomnessRequestInfo",
            (requestRequest as any).info
          );
          if ((requestInfo as any).isLocal) {
            const local = (requestInfo as any).asLocal;
            const requestBlock = local[0];
            return requestBlock.eq(block);
          }
          return false;
        });
        expect(found).is.not.undefined;
      }
    });
  });

  it("should have updated VRF output", async function () {
    this.timeout(10000);

    // we skip on if we aren't past the first block yet
    const notFirstBlock = ((await apiAt.query.randomness.notFirstBlock()) as any).isSome;
    if (notFirstBlock) {
      expect(atBlockNumber).to.be.greaterThan(0); // should be true if notFirstBlock
      const apiAtPrev = await context.polkadotApi.at(
        await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber - 1)
      );

      const currentOutput = await apiAt.query.randomness.localVrfOutput();
      const previousOutput = await apiAtPrev.query.randomness.localVrfOutput();
      expect(currentOutput.eq(previousOutput)).to.be.false;

      // is cleared in on_finalize()
      const inherentIncluded = ((await apiAt.query.randomness.inherentIncluded()) as any).isSome;
      expect(inherentIncluded).to.be.false;
    }
  });
});
