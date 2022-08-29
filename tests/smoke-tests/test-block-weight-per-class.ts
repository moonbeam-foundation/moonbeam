import "@moonbeam-network/api-augment";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:proxy");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

interface BlockWeights {
  hash: string;
  weights: BlockLimits;
}

interface BlockLimits {
  normal: BN;
  operational: BN;
}

describeSmokeSuite(`Verify block weight per class`, { wssUrl, relayWssUrl }, (context) => {
  let blockLimits: BlockLimits;
  let blockWeights: [BlockWeights?] = [];

  before("Retrieve all weight limits and usage", async function () {
    this.timeout(240000);
    // Number of total blocks we want to test
    const batchOf = process.env.BATCH_OF ? parseInt(process.env.BATCH_OF) : 300;
    // Number of blocks to resolve at once
    const concurrency = process.env.CONCURRENCY ? parseInt(process.env.CONCURRENCY) : 10;

    // Promise batch
    const promiseBatch = batchOf / concurrency;

    // Block weight limits per class
    const limits = await context.polkadotApi.consts.system.blockWeights;
    blockLimits = {
      normal: new BN(limits.perClass.normal.maxTotal.toJSON() as number),
      operational: new BN(limits.perClass.operational.maxTotal.toJSON() as number),
    };

    // Best and oldest
    const targetBlock = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    let currentBlock = targetBlock - batchOf - 1;

    // Batch
    for (let i = 0; i < promiseBatch; i++) {
      await Promise.all(
        Array.from(Array(concurrency).keys()).map(async () => {
          currentBlock++;
          const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(currentBlock);
          const apiAt = await context.polkadotApi.at(blockHash);
          const specVersion = (await apiAt.query.system.lastRuntimeUpgrade())
            .unwrap()
            .specVersion.toNumber();
          // RT 1700 introduces CheckWeight in pre_dispatch_self_contained, we only test after
          // https://github.com/paritytech/frontier/pull/749
          if (specVersion >= 1700) {
            const { normal, operational } = await apiAt.query.system.blockWeight();
            blockWeights.push({
              hash: blockHash.toString(),
              weights: {
                normal,
                operational,
              },
            });
          }
        })
      );
    }
    const len = blockWeights.length;
    // Expected length
    expect(len).to.be.eq(batchOf);
    // Expected block hash uniqueness
    expect([...new Set(blockWeights.map((item) => item.hash))].length).to.be.eq(len);
  });

  // Normal class
  it("normal usage should be less than normal dispatch class limits", async function () {
    for (const block of blockWeights) {
      let used = block.weights.normal;
      let allowed = blockLimits.normal;
      expect(used.lte(allowed)).to.be.eq(
        true,
        `${block.hash} normal usage above allowed. Used ${used} and allowed ${allowed}.`
      );
    }
    debug(`Verified normal dispatch class`);
  });
  // Operational class
  it("operational usage should be less than operational dispatch class limits", async function () {
    for (const block of blockWeights) {
      let used = block.weights.operational;
      let allowed = blockLimits.operational;
      expect(used.lte(allowed)).to.be.eq(
        true,
        `${block.hash} operational usage above allowed. Used ${used} and allowed ${allowed}.`
      );
    }
    debug(`Verified operational dispatch class`);
  });
});
