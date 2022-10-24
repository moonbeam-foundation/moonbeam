import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:staking");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

if (!process.env.SKIP_BLOCK_CONSISTENCY_TESTS) {
  describeSmokeSuite(`Verify staking round cleanup`, { wssUrl, relayWssUrl }, function (context) {
    it("storage is cleaned for paid-out rounds", async function () {
      this.timeout(500000);

      const specVersion = context.polkadotApi.consts.system.version.specVersion.toNumber();
      if (specVersion < 1900) {
        debug(`ChainSpec ${specVersion} does not include the storage cleanup, skipping test`);
        this.skip();
      }

      const atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();

      const atBlockHash = await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber);
      const apiAtBlock = await context.polkadotApi.at(atBlockHash);
      const currentRound = (await context.polkadotApi.query.parachainStaking.round()).current;
      const rewardPaymentDelay = context.polkadotApi.consts.parachainStaking.rewardPaymentDelay;
      const lastUnpaidRound = currentRound.sub(rewardPaymentDelay);

      debug(`
  currentRound    ${currentRound.toString()} (#${atBlockNumber} / ${atBlockHash.toString()})
  lastUnpaidRound ${lastUnpaidRound.toString()}`);

      const atStakeInvalidRounds: { [round: number]: number } = {};
      let startKey = "";
      while (true) {
        const result = await apiAtBlock.query.parachainStaking.atStake.keysPaged({
          pageSize: 1000,
          startKey,
          args: [],
        });

        if (result.length === 0) {
          break;
        }
        startKey = result[result.length - 1].toString();
        for (const {
          args: [round, _],
        } of result) {
          if (round < lastUnpaidRound) {
            if (!atStakeInvalidRounds[round.toNumber()]) {
              atStakeInvalidRounds[round.toNumber()] = 0;
            }
            atStakeInvalidRounds[round.toNumber()]++;
          }
        }
      }

      const pointsInvalidRounds: { [round: number]: number } = {};
      while (true) {
        const result = await apiAtBlock.query.parachainStaking.points.keysPaged({
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
          if (round < lastUnpaidRound) {
            if (!pointsInvalidRounds[round.toNumber()]) {
              pointsInvalidRounds[round.toNumber()] = 0;
            }
            pointsInvalidRounds[round.toNumber()]++;
          }
        }
      }

      const delayedPayoutsInvalidRounds: { [round: number]: number } = {};
      while (true) {
        const result = await apiAtBlock.query.parachainStaking.delayedPayouts.keysPaged({
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
          if (round < lastUnpaidRound) {
            if (!delayedPayoutsInvalidRounds[round.toNumber()]) {
              delayedPayoutsInvalidRounds[round.toNumber()] = 0;
            }
            delayedPayoutsInvalidRounds[round.toNumber()]++;
          }
        }
      }

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
}
