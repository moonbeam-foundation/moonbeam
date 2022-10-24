import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:staking");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

if (!process.env.SKIP_BLOCK_CONSISTENCY_TESTS) {
  describeSmokeSuite(`Verify staking snapshot`, { wssUrl, relayWssUrl }, function (context) {
    it("storage is cleaned for paid-out rounds", async function () {
      this.timeout(500000);
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

      const invalidRounds: { [round: number]: number } = {};
      let i = 0;
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
            if (!invalidRounds[round.toNumber()]) {
              invalidRounds[round.toNumber()] = 0;
            }
            invalidRounds[round.toNumber()]++;
          }
        }
      }

      const invalidRoundsCount = Object.keys(invalidRounds).length;
      expect(
        invalidRoundsCount,
        `lastUnpaidRound ${lastUnpaidRound.toString()}, found ${invalidRoundsCount} invalid rounds: ${Object.entries(
          invalidRounds
        ).map(([round, count]) => `${round}(${count})`)}`
      ).to.equal(0);
    });
  });
}
