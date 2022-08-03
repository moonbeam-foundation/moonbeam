import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

// TEMPLATE: Remove useless types at the end
import type { PalletProxyProxyDefinition } from "@polkadot/types/lookup";

// TEMPLATE: Replace debug name
const debug = require("debug")("smoke:randomness");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify number of proxies per account`, { wssUrl, relayWssUrl }, (context) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;

  const requestIds: number[] = [];
  let requestCount: number = 0;

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
        const key = request[0].toString();

        // requestId will be the last 8 bytes (16) nibbles but we need endianness swap
        expect(key.length >= 18, "storage key should be at least 64 bits"); // assumes "0x"
        const subkey = key.slice(-16);
        let idHex = "";
        for (let i = 0; i < subkey.length; i += 2) {
          idHex += subkey.charAt(i + 1);
          idHex += subkey.charAt(i);
        }
        // reverse
        idHex = idHex.split("").reverse().join("");
        const requestId = parseInt(idHex, 16);

        requestIds.push(requestId);
        last_key = key;
      }

      // Debug logs to make sure it keeps progressing
      // TEMPLATE: Adapt log line
      if (true || count % (10 * limit) == 0) {
        debug(`Retrieved ${count} requests`);
        debug(`Array: ${requestIds}`);
      }
    }

    requestCount = ((await apiAt.query.randomness.requestCount()) as any).toNumber();

    // TEMPLATE: Adapt proxies
    debug(`Retrieved ${count} total proxies`);
  });

  it("should have fewer Requests than RequestCount", async function () {
    this.timeout(10000);

    const numOutstandingRequests = requestIds.length;
    expect(numOutstandingRequests).to.be.lessThanOrEqual(requestCount);
  });

  it("should not have requestId above RequestCount", async function () {
    this.timeout(1000);

    const highestId = requestIds.reduce((prev, id) => Math.max(id, prev), 0);
    expect(highestId).to.be.lessThanOrEqual(requestCount);
  });

  it("should not have results without a matching request", async function () {
    this.timeout(10000);

    let query = await apiAt.query.randomness.randomnessResults.entries();
    query.forEach(([key, results]) => {
      console.log(`key: ${key}`);
      /*
      let type = ""; // TODO: type must be reconstructed from the "concat" part of the key
      if (type.isBabeEpoch()) {
        console.log("ignoring babe epoch request"); // TODO
      } else if (type.isLocal()) {
        console.log(`is local: ${type.asLocal()}`);
      }
      */
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
