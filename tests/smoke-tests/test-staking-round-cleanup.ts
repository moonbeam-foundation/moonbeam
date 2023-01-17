import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { QueryableStorageEntry } from "@polkadot/api/types";
import { u32 } from "@polkadot/types";
import type { AccountId20 } from "@polkadot/types/interfaces";

const debug = require("debug")("smoke:staking");

type InvalidRounds = { [round: number]: number };

async function getKeysBeforeRound<
  T extends QueryableStorageEntry<"promise", [u32] | [u32, AccountId20]>
>(lastUnpaidRound: BN, storage: T): Promise<InvalidRounds> {
  const invalidRounds: InvalidRounds = {};
  let startKey = "";
  while (true) {
    const result = await storage.keysPaged({
      pageSize: 1000,
      startKey,
      args: [],
    });

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

describeSmokeSuite("S2100", `Verify staking round cleanup`, function (context, testIt) {
  before(function () {
    if (process.env.SKIP_BLOCK_CONSISTENCY_TESTS) {
      debug("Skip Block Consistency flag set, skipping staking round cleanup tests.");
      this.skip();
    }
  });

  testIt("C100", `storage is cleaned for paid-out rounds`, async function () {
    this.timeout(500000);

    const specVersion = context.polkadotApi.consts.system.version.specVersion.toNumber();
    if (specVersion < 2000) {
      debug(`ChainSpec ${specVersion} does not include the storage cleanup, skipping test`);
      this.skip();
    }
    const currentBlock = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    if (currentBlock < 1000) {
      debug(`Current block is < 1000 (probably for Fork test), skipping test`);
      this.skip();
    }

    const atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : currentBlock;

    const atBlockHash = await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber);
    const apiAtBlock = await context.polkadotApi.at(atBlockHash);
    const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
    const rewardPaymentDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
    const lastUnpaidRound = currentRound.sub(rewardPaymentDelay);

    debug(`
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
  });
});
