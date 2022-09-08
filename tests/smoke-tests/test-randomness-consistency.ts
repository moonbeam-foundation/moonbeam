import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { BN, hexToBigInt } from "@polkadot/util";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:randomness");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

const RANDOMNESS_ACCOUNT_ID = "0x6d6f646c6d6f6f6e72616e640000000000000000";

describeSmokeSuite(`Verify randomness consistency`, { wssUrl, relayWssUrl }, (context) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;

  const requestStates: { id: number; state: any }[] = [];
  let numRequests: number = 0; // our own count
  let requestCount: number = 0; // from pallet storage

  let isRandomnessAvailable = true;

  before("Retrieve all requests", async function () {
    this.timeout(30_000); // 30s

    const runtimeVersion = context.polkadotApi.runtimeVersion.specVersion.toNumber();
    const runtimeName = context.polkadotApi.runtimeVersion.specName.toString();
    isRandomnessAvailable =
      (runtimeVersion >= 1700 && runtimeName == "moonbase") || runtimeVersion >= 1900;

    if (!isRandomnessAvailable) {
      debug(`Skipping test [RT${runtimeVersion} ${runtimeName}]`);
      return;
    }

    const limit = 1000;
    let last_key = "";
    let count = 0;

    atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );

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

      for (const request of query) {
        const key = request[0].toHex();
        expect(key.length >= 18, "storage key should be at least 64 bits"); // assumes "0x"

        const requestIdEncoded = key.slice(-16);
        const requestId = hexToBigInt(requestIdEncoded, { isLe: true });

        requestStates.push({ id: Number(requestId), state: request[1] });
        numRequests += 1;
        last_key = key;
      }

      if (true || count % (10 * limit) == 0) {
        debug(`Retrieved ${count} requests`);
        debug(`Requests: ${requestStates}`);
      }
    }

    requestCount = ((await apiAt.query.randomness.requestCount()) as any).toNumber();

    debug(`Retrieved ${count} total requests`);
  });

  it("should have fewer Requests than RequestCount", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

    const numOutstandingRequests = numRequests;
    expect(numOutstandingRequests).to.be.lessThanOrEqual(requestCount);
  });

  it("should not have requestId above RequestCount", async function () {
    this.timeout(1000);

    if (!isRandomnessAvailable) {
      return;
    }

    const highestId = requestStates.reduce((prev, request) => Math.max(request.id, prev), 0);
    expect(highestId).to.be.lessThanOrEqual(requestCount);
  });

  it("should not have results without a matching request", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

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
        let found = requestStates.find((request) => {
          // TODO: can we traverse this hierarchy of types without creating each?
          const requestState = context.polkadotApi.registry.createType(
            "PalletRandomnessRequestState",
            request.state.toHex()
          );
          const requestRequest = context.polkadotApi.registry.createType(
            "PalletRandomnessRequest",
            (requestState as any).request.toHex()
          );
          const requestInfo = context.polkadotApi.registry.createType(
            "PalletRandomnessRequestInfo",
            (requestRequest as any).info
          );
          if ((requestInfo as any).isBabeEpoch) {
            const babe = (requestInfo as any).asBabeEpoch;
            const requestEpoch = babe[0];
            return requestEpoch.eq(epoch);
          }
          return false;
        });
        expect(found).is.not.undefined;
      } else {
        // look for any requests which depend on the "local" block
        let block = (requestType as any).asLocal;
        let found = requestStates.find((request) => {
          // TODO: can we traverse this hierarchy of types without creating each?
          const requestState = context.polkadotApi.registry.createType(
            "PalletRandomnessRequestState",
            request.state.toHex()
          );
          const requestRequest = context.polkadotApi.registry.createType(
            "PalletRandomnessRequest",
            (requestState as any).request.toHex()
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

  it("all results should have correct request counters", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

    // Local count for request types
    const requestCounts = {};
    requestStates.forEach((request) => {
      const requestState = context.polkadotApi.registry.createType(
        "PalletRandomnessRequestState",
        request.state.toHex()
      );
      const requestRequest = context.polkadotApi.registry.createType(
        "PalletRandomnessRequest",
        (requestState as any).request.toHex()
      );
      const requestInfo = context.polkadotApi.registry.createType(
        "PalletRandomnessRequestInfo",
        (requestRequest as any).info
      );
      if ((requestInfo as any).isBabeEpoch) {
        const babe = (requestInfo as any).asBabeEpoch;
        requestCounts[babe[0]] = (requestCounts[babe[0]] || new BN(0)).add(new BN(1));
      } else {
        const local = (requestInfo as any).asLocal;
        requestCounts[local[0]] = (requestCounts[local[0]] || new BN(0)).add(new BN(1));
      }
    });
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
      const result = context.polkadotApi.registry.createType(
        "PalletRandomnessRandomnessResult",
        results.toHex()
      );
      const resultRequestCount = (result as any).requestCount;
      if ((requestType as any).isBabeEpoch) {
        let epoch = (requestType as any).asBabeEpoch;
        expect(requestCounts[epoch].toString()).to.equal(
          resultRequestCount.toString(),
          `Counted request count ${requestCounts[epoch]} != ${resultRequestCount} for result:\n` +
            `${result}`
        );
      } else {
        let local = (requestType as any).asLocal;
        expect(requestCounts[local].toString()).to.equal(
          resultRequestCount.toString(),
          `Counted request count ${requestCounts[local]} != ${resultRequestCount} for result:\n` +
            `${result}`
        );
      }
    });
  });

  it("should have updated VRF output", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

    // we skip on if we aren't past the first block yet
    const notFirstBlock = ((await apiAt.query.randomness.notFirstBlock()) as any).isSome;
    if (notFirstBlock) {
      expect(atBlockNumber).to.be.greaterThan(0); // should be true if notFirstBlock
      const apiAtPrev = await context.polkadotApi.at(
        await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber - 1)
      );

      const currentOutput = await apiAt.query.randomness.localVrfOutput();
      const previousOutput = await apiAtPrev.query.randomness.localVrfOutput();
      const currentVrfOutput = context.polkadotApi.registry.createType(
        "Option<H256>",
        (currentOutput as any).toHex()
      );
      const previousVrfOutput = context.polkadotApi.registry.createType(
        "Option<H256>",
        (previousOutput as any).toHex()
      );
      expect(previousVrfOutput.isSome).to.equal(
        true,
        `Previous local VRF output must always be inserted into storage but isNone`
      );
      expect(currentVrfOutput.isSome).to.equal(
        true,
        `Current local VRF output must always be inserted into storage but isNone`
      );
      expect(currentVrfOutput.unwrap().eq(previousVrfOutput.unwrap())).to.be.false;

      // is cleared in on_finalize()
      const inherentIncluded = ((await apiAt.query.randomness.inherentIncluded()) as any).isSome;
      expect(inherentIncluded).to.be.false;
    }
  });

  it("should have correct total deposits", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

    let totalDeposits = 0n;
    for (const request of requestStates) {
      // TODO: copied from above -- this could use some DRY
      const requestState = context.polkadotApi.registry.createType(
        "PalletRandomnessRequestState",
        request.state.toHex()
      );
      const requestRequest = context.polkadotApi.registry.createType(
        "PalletRandomnessRequest",
        (requestState as any).request.toHex()
      );

      totalDeposits += BigInt((requestRequest as any).fee);
      totalDeposits += BigInt((requestState as any).deposit);
    }

    const palletAccountBalance = (
      await apiAt.query.system.account(RANDOMNESS_ACCOUNT_ID)
    ).data.free.toBigInt();

    expect(palletAccountBalance >= totalDeposits).to.be.true;
  });

  it("available randomness outputs should be random", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

    let query = await apiAt.query.randomness.randomnessResults.entries();
    await query.forEach(([key, results]) => {
      const result = context.polkadotApi.registry.createType(
        "PalletRandomnessRandomnessResult",
        results.toHex()
      );
      const randomnessResult = (result as any).randomness;
      if (randomnessResult.isSome) {
        isRandom(randomnessResult.unwrap());
      }
    });
  });

  it("local VRF output should be random", async function () {
    this.timeout(10000);

    if (!isRandomnessAvailable) {
      return;
    }

    const notFirstBlock = ((await apiAt.query.randomness.notFirstBlock()) as any).isSome;
    if (notFirstBlock) {
      const currentOutput = await apiAt.query.randomness.localVrfOutput();
      const currentRawOutput = context.polkadotApi.registry.createType(
        "H256",
        (currentOutput as any).toHex()
      );
      isRandom(currentRawOutput);
    }
  });
});

// Tests whether the input bytes appear to be random by measuring the distribution relative to
// what would be expected of a uniformly distributed [u8; 32]
function isRandom(bytes: Uint8Array) {
  // test whether output bytes are statistically independent
  chiSquareTest(bytes);
  // expect average byte of [u8; 32] = ~128 if uniformly distributed ~> expect 81 < X < 175
  averageByteWithinExpectedRange(bytes, 81, 175);
  // expect fewer than 4 repeated values in output [u8; 32]
  outputWithinExpectedRepetition(bytes, 3);
}

// Tests if byte output is independent
function chiSquareTest(bytes: Uint8Array) {
  let chiSquared = 0.0;
  let numOnes = 0;
  // expected value of 256 coin flips:
  const expectedValue = 256 / 2;
  // degrees of freedom is 256 - 1 = 255, alpha is 0.05
  // chi.pdf(250, 0.05) = 287.882 (TODO: use precise value; this is from 250 in following table)
  // https://en.wikibooks.org/wiki/Engineering_Tables/Chi-Squared_Distibution

  // count occurences of ones
  const pValue = 286.882;
  bytes.forEach((a) => {
    // convert to a binary text string, e.g. "101101", then count the ones.
    // note that this string excluded insignificant digits, which is why we count ones
    numOnes += [...a.toString(2)].filter((x) => x === "1").length;
  });

  chiSquared += (numOnes - expectedValue) ** 2.0 / expectedValue;
  let numZeroes = 256 - numOnes;
  chiSquared += (numZeroes - expectedValue) ** 2.0 / expectedValue;

  expect(numOnes + numZeroes).to.equal(256, "Data should produce exactly 256 bits");

  expect(chiSquared < pValue).to.equal(
    true,
    `Chi square value greater than or equal to expected so bytes in output appear related` +
      `chiSquared is ${chiSquared} >= ${pValue}`
  );
}

// Tests uniform distribution of outputs bytes by checking if average byte is within expected range
function averageByteWithinExpectedRange(bytes: Uint8Array, min: number, max: number) {
  const average = bytes.reduce((a, b) => a + b) / bytes.length;
  expect(min <= average && average <= max).to.equal(true, `Average bytes is ${average}`);
}

// Tests uniform distribution of outputs bytes by checking if any repeated bytes
function outputWithinExpectedRepetition(bytes: Uint8Array, maxRepeats: number) {
  const counts = {};
  let fewerThanMaxRepeats = true;
  bytes.forEach(function (x) {
    let newCount: number = (counts[x] || 0) + 1;
    counts[x] = newCount;
    if (newCount > maxRepeats) {
      debug(`Count of ${x} > ${maxRepeats} maxRepeats\n` + `Bytes: ${bytes}`);
      fewerThanMaxRepeats = false;
    }
  });
  expect(fewerThanMaxRepeats).to.be.true;
}
