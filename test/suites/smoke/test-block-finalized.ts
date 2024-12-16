import "@polkadot/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import {
  checkBlockFinalized,
  getBlockTime,
  fetchHistoricBlockNum,
  TEN_MINS,
  TWO_HOURS,
} from "@moonwall/util";
import semver from "semver";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import type { Signer } from "ethers";
import type { ApiPromise } from "@polkadot/api";
import { rateLimiter } from "../../helpers/common.js";

const timePeriod = process.env.TIME_PERIOD ? Number(process.env.TIME_PERIOD) : TWO_HOURS;
const timeout = Math.floor(timePeriod / 12); // 2 hour -> 10 minute timeout

describeSuite({
  id: "S04",
  title: "Parachain blocks should be finalized",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let paraApi: ApiPromise;
    let ethers: Signer;

    beforeAll(() => {
      paraApi = context.polkadotJs("para");
      ethers = context.ethers()!;
    });

    it({
      id: "C100",
      title: "should have a recently finalized block",
      test: async function () {
        const head = await paraApi.rpc.chain.getFinalizedHead();
        const block = await paraApi.rpc.chain.getBlock(head);

        const diff = Date.now() - getBlockTime(block);
        log(`Last finalized block was ${diff / 1000} seconds ago`);
        expect(diff).to.be.lessThanOrEqual(TEN_MINS);
      },
    });

    it({
      id: "C200",
      title: "should have a recently finalized eth block",
      test: async function () {
        const specVersion = paraApi.consts.system.version.specVersion.toNumber();
        const clientVersion = (await paraApi.rpc.system.version()).toString().split("-")[0];

        if (specVersion < 1900 || semver.lt(clientVersion, "0.27.2")) {
          log(`ChainSpec ${specVersion}, client ${clientVersion} unsupported BlockTag, skipping.`);
          return; // TODO: replace with skip() when added to vitest
        }

        const timestamp = (await ethers.provider!.getBlock("finalized"))!.timestamp;
        const diff = Date.now() - timestamp * 1000;
        log(`Last finalized eth block was ${diff / 1000} seconds ago`);
        expect(diff).to.be.lessThanOrEqual(10 * 60 * 1000);
      },
    });

    it({
      id: "C300",
      title:
        "should have only finalized blocks in the past" +
        ` ${(timePeriod / (1000 * 60 * 60)).toFixed(2)} hours #C300`,
      timeout: timeout,
      test: async function () {
        const signedBlock = await paraApi.rpc.chain.getBlock(
          await paraApi.rpc.chain.getFinalizedHead()
        );
        const lastBlockNumber = signedBlock.block.header.number.toNumber();
        const lastBlockTime = getBlockTime(signedBlock);
        const limiter = rateLimiter();

        const firstBlockTime = lastBlockTime - timePeriod;
        log(`Searching for the block at: ${new Date(firstBlockTime)}`);

        const firstBlockNumber = (await limiter.wrap(fetchHistoricBlockNum)(
          paraApi,
          lastBlockNumber,
          firstBlockTime
        )) as number;

        log(`Checking if blocks #${firstBlockNumber} - #${lastBlockNumber} are finalized.`);

        const promises = (() => {
          const length = lastBlockNumber - firstBlockNumber;
          return Array.from({ length }, (_, i) => firstBlockNumber + i);
        })().map((num) => limiter.schedule(() => checkBlockFinalized(paraApi, num)));

        const results = await Promise.all(promises);

        const unfinalizedBlocks = results.filter((item) => !item.finalized);
        expect(
          unfinalizedBlocks,
          `The following blocks were not finalized ${unfinalizedBlocks
            .map((a) => a.number)
            .join(", ")}`
        ).to.be.empty;
      },
    });
  },
});
