import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:ethereum-mapping");

// At rpc-level, `*ByNumber` requests always use the canonical block reference given by Substrate.
// In the other hand `*ByHash` requests rely on data mapped in the frontier db.
// We want to compare both to verify recent db data consistency and rpc impl across client versions.

describeSmokeSuite(
  "S800",
  `Ethereum secondary DB should contains valid data `,
  (context, testIt) => {
    testIt(
      "C100",
      `should get the same response payload on byNumber and byHash requests`,
      async function () {
        this.timeout(60_000); // 1 minute
        // As we are testing rpc-level functionality the height at which we access secondary db data
        // is irrelevant. We can just select some arbitrary block numbers to verify block hashes.
        const latestBlockNumber = await context.ethers.getBlockNumber();
        // We asume we only want to run the test if there is enough blocks.
        if (latestBlockNumber > 10000) {
          let failedCheckpoints = [];

          const checkPoint_1 = latestBlockNumber - 10;
          const checkPoint_2 = latestBlockNumber - 100;
          const checkPoint_3 = latestBlockNumber - 1000;
          const checkPoint_4 = latestBlockNumber - 10000;

          const blocks = [
            latestBlockNumber,
            checkPoint_1,
            checkPoint_2,
            checkPoint_3,
            checkPoint_4,
          ];

          for (const block of blocks) {
            const byNumber = await context.ethers.getBlock(block);
            const byHash = await context.ethers.getBlock(byNumber.hash);
            if (JSON.stringify(byNumber) !== JSON.stringify(byHash)) {
              failedCheckpoints.push(block);
            }
          }
          expect(failedCheckpoints.length).to.be.eq(
            0,
            `Inconsistency found at ${JSON.stringify(failedCheckpoints)}`
          );
        }
      }
    );
  }
);
