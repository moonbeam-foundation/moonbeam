import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { BN } from "@polkadot/util";
import { QueryableStorageEntry } from "@polkadot/api/types";
import { u32 } from "@polkadot/types";
import type { AccountId20 } from "@polkadot/types/interfaces";
import { ApiPromise } from "@polkadot/api";
import { TEN_MINS } from "@moonwall/util";
import { rateLimiter } from "../../helpers/common.js";
const limiter = rateLimiter();

type InvalidRounds = { [round: number]: number };

async function getKeysBeforeRound<
  T extends QueryableStorageEntry<"promise", [u32] | [u32, AccountId20]>
>(lastUnpaidRound: BN, storage: T): Promise<InvalidRounds> {
  const invalidRounds: InvalidRounds = {};
  let startKey = "";
  for (;;) {
    const result = await limiter.schedule(() =>
      storage.keysPaged({
        pageSize: 1000,
        startKey,
        args: [],
      })
    );
    if (result.length === 0) {
      break;
    }
    startKey = result[result.length - 1].toString();
    for (const {
      args: [round],
    } of result) {
      if (round.lt(lastUnpaidRound)) {
        if (!invalidRounds[round.toNumber()]) {
          invalidRounds[round.toNumber()] = 0;
        }
        invalidRounds[round.toNumber()]++;
      }
    }
  }
  return invalidRounds;
}

describeSuite({
  id: "S2100",
  title: "Verify staking round cleanup",
  foundationMethods: "read_only",
  testCases: function ({ context, it, log }) {
    let paraApi: ApiPromise;

    beforeAll(function () {
      paraApi = context.polkadotJs("para");
    });

    it({
      id: "C100",
      title: "storage is cleaned for paid-out rounds",
      timeout: TEN_MINS,
      test: async function () {
        const specVersion = paraApi.consts.system.version.specVersion.toNumber();
        if (specVersion < 2000) {
          log(`ChainSpec ${specVersion} does not include the storage cleanup, skipping test`);
          return;
        }
        const currentBlock = (await paraApi.rpc.chain.getHeader()).number.toNumber();
        if (currentBlock < 1000) {
          log(`Current block is < 1000 (probably for Fork test), skipping test`);
          return;
        }

        const atBlockNumber = process.env.BLOCK_NUMBER
          ? parseInt(process.env.BLOCK_NUMBER)
          : currentBlock;

        const atBlockHash = await paraApi.rpc.chain.getBlockHash(atBlockNumber);
        const apiAtBlock = await paraApi.at(atBlockHash);
        const currentRound = (await paraApi.query.parachainStaking.round()).current;
        const rewardPaymentDelay = paraApi.consts.parachainStaking.rewardPaymentDelay;
        const lastUnpaidRound = currentRound.sub(rewardPaymentDelay);

        log(`
  currentRound    ${currentRound.toString()} (#${atBlockNumber} / ${atBlockHash.toString()})
  lastUnpaidRound ${lastUnpaidRound.toString()}`);

        const awardedPtsInvalidRounds = await getKeysBeforeRound(
          lastUnpaidRound,
          apiAtBlock.query.parachainStaking.awardedPts
        );

        const pointsInvalidRounds = await getKeysBeforeRound(
          lastUnpaidRound,
          apiAtBlock.query.parachainStaking.points
        );
        const delayedPayoutsInvalidRounds = await getKeysBeforeRound(
          lastUnpaidRound,
          apiAtBlock.query.parachainStaking.delayedPayouts
        );
        const atStakeInvalidRounds = await getKeysBeforeRound(
          lastUnpaidRound,
          apiAtBlock.query.parachainStaking.atStake
        );

        const awardedPtsInvalidRoundsCount = Object.keys(awardedPtsInvalidRounds).length;
        expect(
          awardedPtsInvalidRoundsCount,
          `[AwardedPts] lastUnpaidRound ${lastUnpaidRound.toString()},\
        found ${awardedPtsInvalidRoundsCount} invalid rounds: \
        ${Object.entries(awardedPtsInvalidRounds).map(([round, count]) => `${round}(${count})`)}`
        ).to.equal(0);

        const pointsInvalidRoundsCount = Object.keys(pointsInvalidRounds).length;
        expect(
          pointsInvalidRoundsCount,
          `[Points] lastUnpaidRound ${lastUnpaidRound.toString()},\
        found ${pointsInvalidRoundsCount} invalid rounds: \
        ${Object.entries(pointsInvalidRounds).map(([round, count]) => `${round}(${count})`)}`
        ).to.equal(0);

        const delayedPayoutsInvalidRoundsCount = Object.keys(delayedPayoutsInvalidRounds).length;
        expect(
          delayedPayoutsInvalidRoundsCount,
          `[DelayedPayouts] lastUnpaidRound ${lastUnpaidRound.toString()},\
        found ${delayedPayoutsInvalidRoundsCount} invalid rounds: \
        ${Object.entries(delayedPayoutsInvalidRounds).map(
          ([round, count]) => `${round}(${count})`
        )}`
        ).to.equal(0);

        const atStakeInvalidRoundsCount = Object.keys(atStakeInvalidRounds).length;
        expect(
          atStakeInvalidRoundsCount,
          `[AtStake] lastUnpaidRound ${lastUnpaidRound.toString()},\
        found ${atStakeInvalidRoundsCount} invalid rounds: \
        ${Object.entries(atStakeInvalidRounds).map(([round, count]) => `${round}(${count})`)}`
        ).to.equal(0);
      },
    });
  },
});
