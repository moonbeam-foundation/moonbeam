import "@moonbeam-network/api-augment";
import type { ApiDecoration } from "@polkadot/api/types";
import { BN, hexToBigInt } from "@polkadot/util";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import randomLib from "randomness";
import type { Bit } from "randomness/lib/types";
import chalk from "chalk";

const RANDOMNESS_ACCOUNT_ID = "0x6d6f646c6d6f6f6e72616e640000000000000000";

describeSuite({
  id: "S18",
  title: "Verify randomness consistency",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber = 0;
    let apiAt: ApiDecoration<"promise">;
    const requestStates: { id: number; state: any }[] = [];
    let numRequests = 0; // our own count
    let requestCount = 0; // from pallet storage
    let isRandomnessAvailable = true;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const runtimeVersion = paraApi.runtimeVersion.specVersion.toNumber();
      const runtimeName = paraApi.runtimeVersion.specName.toString();
      isRandomnessAvailable =
        (runtimeVersion >= 1700 && runtimeName === "moonbase") || runtimeVersion >= 1900;

      if (!isRandomnessAvailable) {
        log(`Skipping test [RT${runtimeVersion} ${runtimeName}]`);
        return;
      }

      const limit = 1000;
      let last_key = "";
      let count = 0;

      atBlockNumber = process.env.BLOCK_NUMBER
        ? Number.parseInt(process.env.BLOCK_NUMBER)
        : (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));

      for (;;) {
        const query = await apiAt.query.randomness.requests.entriesPaged({
          args: [],
          pageSize: limit,
          startKey: last_key,
        });

        if (query.length === 0) {
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

        if (count % (10 * limit) === 0) {
          log(`Retrieved ${count} requests`);
          log(`Requests: ${requestStates.map((r) => r.id).join(",")}`);
        }
      }

      requestCount = ((await apiAt.query.randomness.requestCount()) as any).toNumber();

      log(`Retrieved ${count} total requests`);
    }, 30_000);

    it({
      id: "C100",
      title: "should have fewer Requests than RequestCount",
      timeout: 10000,
      test: async function () {
        if (!isRandomnessAvailable) {
          return;
        }

        const numOutstandingRequests = numRequests;
        expect(numOutstandingRequests).to.be.lessThanOrEqual(requestCount);
      },
    });

    it({
      id: "C200",
      title: "should not have requestId above RequestCount",
      test: async function () {
        if (!isRandomnessAvailable) {
          return;
        }

        const highestId = requestStates.reduce((prev, request) => Math.max(request.id, prev), 0);
        expect(highestId).to.be.lessThanOrEqual(requestCount);
      },
    });

    it({
      id: "C300",
      title: "should not have results without a matching request",
      timeout: 10000,
      test: async function () {
        if (!isRandomnessAvailable) {
          return;
        }

        const query = await apiAt.query.randomness.randomnessResults.entries();
        query.forEach(([key, results]) => {
          // offset is:
          // * 2 for "0x"
          // * 32 for module
          // * 32 for method
          // * 16 for the hashed part of the key: the twox64(someRequestType) part
          // the remaining substr after offset is the concat part,
          // which we can decode with createType
          const offset = 2 + 32 + 32 + 16;
          const requestTypeEncoded = key.toHex().slice(offset);
          const requestType = paraApi.registry.createType(
            `PalletRandomnessRequestType`,
            "0x" + requestTypeEncoded
          );

          // sanity check
          expect(
            (requestType as any).isBabeEpoch || (requestType as any).isLocal,
            "unexpected enum in encoded RequestType string"
          );

          if ((requestType as any).isBabeEpoch) {
            const epoch = (requestType as any).asBabeEpoch;
            const found = requestStates.find((request) => {
              // TODO: can we traverse this hierarchy of types without creating each?
              const requestState = paraApi.registry.createType(
                "PalletRandomnessRequestState",
                request.state.toHex()
              );
              const requestRequest = paraApi.registry.createType(
                "PalletRandomnessRequest",
                (requestState as any).request.toHex()
              );
              const requestInfo = paraApi.registry.createType(
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
            const block = (requestType as any).asLocal;
            const found = requestStates.find((request) => {
              // TODO: can we traverse this hierarchy of types without creating each?
              const requestState = paraApi.registry.createType(
                "PalletRandomnessRequestState",
                request.state.toHex()
              );
              const requestRequest = paraApi.registry.createType(
                "PalletRandomnessRequest",
                (requestState as any).request.toHex()
              );
              const requestInfo = paraApi.registry.createType(
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
      },
    });

    it({
      id: "C400",
      title: "all results should have correct request counters",
      timeout: 10000,
      test: async function () {
        if (!isRandomnessAvailable) {
          return;
        }

        // Local count for request types
        const requestCounts: any = {};
        requestStates.forEach((request) => {
          const requestState = paraApi.registry.createType(
            "PalletRandomnessRequestState",
            request.state.toHex()
          );
          const requestRequest = paraApi.registry.createType(
            "PalletRandomnessRequest",
            (requestState as any).request.toHex()
          );
          const requestInfo = paraApi.registry.createType(
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
        const query = await apiAt.query.randomness.randomnessResults.entries();
        query.forEach(([key, results]) => {
          // offset is:
          // * 2 for "0x"
          // * 32 for module
          // * 32 for method
          // * 16 for the hashed part of the key: the twox64(someRequestType) part
          // the remaining substr after offset is the concat part,
          // which we can decode with createType
          const offset = 2 + 32 + 32 + 16;
          const requestTypeEncoded = key.toHex().slice(offset);
          const requestType = paraApi.registry.createType(
            `PalletRandomnessRequestType`,
            "0x" + requestTypeEncoded
          );
          const result = paraApi.registry.createType(
            "PalletRandomnessRandomnessResult",
            results.toHex()
          );
          const resultRequestCount = (result as any).requestCount;
          if ((requestType as any).isBabeEpoch) {
            const epoch = (requestType as any).asBabeEpoch;
            expect(requestCounts[epoch].toString()).to.equal(
              resultRequestCount.toString(),
              "Counted request count" +
                `${requestCounts[epoch]} != ${resultRequestCount} for result:\n` +
                `${result}`
            );
          } else {
            const local = (requestType as any).asLocal;
            expect(requestCounts[local].toString()).to.equal(
              resultRequestCount.toString(),
              "Counted request count" +
                `${requestCounts[local]} != ${resultRequestCount} for result:\n` +
                `${result}`
            );
          }
        });
      },
    });

    it({
      id: "C500",
      title: "should have updated VRF output",
      timeout: 10000,
      test: async function () {
        if (!isRandomnessAvailable) {
          return;
        }

        // we skip on if we aren't past the first block yet
        const notFirstBlock = ((await apiAt.query.randomness.notFirstBlock()) as any).isSome;
        if (notFirstBlock) {
          expect(atBlockNumber).to.be.greaterThan(0); // should be true if notFirstBlock
          const apiAtPrev = await paraApi.at(
            await paraApi.rpc.chain.getBlockHash(atBlockNumber - 1)
          );

          const currentOutput = await apiAt.query.randomness.localVrfOutput();
          const previousOutput = await apiAtPrev.query.randomness.localVrfOutput();
          const currentVrfOutput = paraApi.registry.createType(
            "Option<H256>",
            (currentOutput as any).toHex()
          );
          const previousVrfOutput = paraApi.registry.createType(
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
          const inherentIncluded = ((await apiAt.query.randomness.inherentIncluded()) as any)
            .isSome;
          expect(inherentIncluded).to.be.false;
        }
      },
    });

    it({
      id: "C600",
      title: "should have correct total deposits",
      timeout: 10000,
      test: async function () {
        if (!isRandomnessAvailable) {
          return;
        }

        let totalDeposits = 0n;
        for (const request of requestStates) {
          // TODO: copied from above -- this could use some DRY
          const requestState = paraApi.registry.createType(
            "PalletRandomnessRequestState",
            request.state.toHex()
          );
          const requestRequest = paraApi.registry.createType(
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
      },
    });

    it({
      id: "C700",
      title: "available randomness outputs should be random",
      timeout: 10000,
      test: async function () {
        // We are using the NIST guideline thresholds, however we are only really concerned if
        // multiple tests fail given these are all probabilistic tests
        const maxTestFailures = 4;

        if (!isRandomnessAvailable) {
          return;
        }

        const query = await apiAt.query.randomness.randomnessResults.entries();
        const randomTestResults = query.map(([key, results]) => {
          const formattedKey = (key.toHuman() as any)[0];
          return {
            request: Object.values(formattedKey)[0],
            requestType: Object.keys(formattedKey)[0],
            testResults: results.isSome ? isRandom(results.unwrap().randomness.toU8a()) : true,
          };
        });

        const failures = randomTestResults
          .map((item) => {
            const request = `${item.requestType}: ${(item.request as string).replaceAll(",", "")}`;
            return {
              request,
              failures: Object.entries(item.testResults).reduce((acc, curr) => {
                if (!curr[1]) {
                  log(`${chalk.bgBlack.greenBright(curr[0])} failed for request ${request}`);
                  return acc + 1;
                }
                return acc;
              }, 0),
            };
          })
          .filter((result) => result.failures > maxTestFailures);

        expect(
          failures.length,
          `${maxTestFailures}+ randomness checks failed for: ${failures
            .map((fail) => fail.request)
            .join(", ")}.`
        ).to.equal(0);
      },
    });

    it({
      id: "C800",
      title: "local VRF output should be random",
      timeout: 10000,
      test: async function () {
        // We are using the NIST guideline thresholds, however we are only really concerned if
        // multiple tests fail given these are all probabilistic tests
        const maxTestFailures = 4;

        if (!isRandomnessAvailable) {
          return;
        }

        if (!(await apiAt.query.randomness.notFirstBlock()).isSome) {
          log(`This is first block (genesis/runtime upgrade) so skipping test`);
          return;
        }

        const currentOutput = await apiAt.query.randomness.localVrfOutput();
        const randomTestResults = isRandom(currentOutput.unwrapOrDefault().toU8a());
        const failures = Object.entries(randomTestResults).filter(([_, result]) => !result);

        expect(
          failures.length,
          `Failed random at #${atBlockNumber} for local VRF: ${failures
            .map((test) => test[0])
            .join(", ")}`
        ).toBeLessThan(maxTestFailures);
      },
    });

    // The tests here have been taken from recommendations of the NIST whitepaper on
    // "A Statistical Test Suite for Random and Pseudorandom Number Generators
    // for Cryptographic Applications" - Lawrence E Bassham III (2010)
    // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-22r1a.pdf
    function isRandom(bytes: Uint8Array) {
      const binaryArray = uint8ArrayToBinaryArray(bytes);
      return {
        approximateEntropyTest: randomLib.approximateEntropyTest(binaryArray)[0],
        cumulativeSumsTest: randomLib.cumulativeSumsTest(binaryArray)[0],
        frequencyWithinBlockTest: randomLib.frequencyWithinBlockTest(binaryArray)[0],
        longestRunOnesInABlockTest: randomLib.longestRunOnesInABlockTest(binaryArray)[0],
        monobitTest: randomLib.monobitTest(binaryArray)[0],
        runsTest: randomLib.runsTest(binaryArray)[0],

        // In-house randomness tests
        chiSquareTest: chiSquareTest(bytes),
        // expect average byte of [u8; 32] = ~128 if uniformly distributed ~> expect 81 < X < 175
        averageByteWithinExpectedRange: averageByteWithinExpectedRange(bytes, 81, 175),
        // expect fewer than 4 repeated values in output [u8; 32]
        outputWithinExpectedRepetition: outputWithinExpectedRepetition(bytes, 3),
      };
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
      const numZeroes = 256 - numOnes;
      chiSquared += (numZeroes - expectedValue) ** 2.0 / expectedValue;

      //Data should produce exactly 256 bits
      const lengthCheck = numOnes + numZeroes === 256;

      // Chi square value greater than or equal to expected so bytes in output appear related` +
      const chiCheck = chiSquared < pValue;
      return lengthCheck && chiCheck;
    }

    // Tests uniform distribution of outputs bytes by checking if average byte is in expected range
    function averageByteWithinExpectedRange(bytes: Uint8Array, min: number, max: number) {
      const average = bytes.reduce((a, b) => a + b) / bytes.length;
      return min <= average && average <= max;
    }

    // Tests uniform distribution of outputs bytes by checking if any repeated bytes
    function outputWithinExpectedRepetition(bytes: Uint8Array, maxRepeats: number) {
      const counts = bytes.reduce(
        (acc, byte) => {
          acc[byte] = (acc[byte] || 0) + 1;
          return acc;
        },
        {} as { [byte: string]: number }
      );

      const exceededRepeats = Object.values(counts).some((count) => count > maxRepeats);

      if (exceededRepeats) {
        const problematicByte = Object.keys(counts).find((byte) => counts[byte] > maxRepeats);
        log(
          `Count of ${problematicByte}: ${counts[problematicByte!]} > ${maxRepeats} maxRepeats\n` +
            `Bytes: ${Array.from(bytes).toString()}`
        );
      }
      return !exceededRepeats;
    }
  },
});

function uint8ArrayToBinaryArray(data: Uint8Array): Bit[] {
  const result: number[] = [];
  data.forEach((byte) => {
    for (let i = 7; i >= 0; i--) {
      result.push((byte >> i) & 1);
    }
  });
  return result as Bit[];
}
