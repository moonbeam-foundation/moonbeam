import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { BN } from "@polkadot/util";
import { u128, u32 } from "@polkadot/types";
import { ApiPromise } from "@polkadot/api";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:staking");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

if (!process.env.SKIP_BLOCK_CONSISTENCY_TESTS) {
  describeSmokeSuite(`Verify staking rewards`, { wssUrl, relayWssUrl }, function (context) {
    it("rewards are given as expected", async function () {
      this.timeout(500000);
      const atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
      await assertRewardsAtRoundBefore(context.polkadotApi, atBlockNumber);
    });
  });
}

async function assertRewardsAtRoundBefore(api: ApiPromise, nowBlockNumber: number) {
  const nowBlockHash = await api.rpc.chain.getBlockHash(nowBlockNumber);
  const nowRound = await (await api.at(nowBlockHash)).query.parachainStaking.round();
  const previousRoundBlock = nowRound.first.subn(1).toNumber();

  await assertRewardsAt(api, previousRoundBlock);
}

async function assertRewardsAt(api: ApiPromise, nowBlockNumber: number) {
  const nowBlockHash = await api.rpc.chain.getBlockHash(nowBlockNumber);
  const nowRound = await (await api.at(nowBlockHash)).query.parachainStaking.round();
  const nowRoundNumber = nowRound.current;
  const nowRoundFirstBlock = nowRound.first;
  const nowRoundFirstBlockHash = await api.rpc.chain.getBlockHash(nowRoundFirstBlock);
  const apiAtRewarded = await api.at(nowRoundFirstBlockHash);
  const rewardDelay = apiAtRewarded.consts.parachainStaking.rewardPaymentDelay;
  const priorRewardedBlockHash = await api.rpc.chain.getBlockHash(nowRoundFirstBlock.subn(1));
  const _specVersion = (await apiAtRewarded.query.system.lastRuntimeUpgrade())
    .unwrap()
    .specVersion.toNumber();

  // obtain data from original round
  const originalRoundNumber = (await apiAtRewarded.query.parachainStaking.round()).current.sub(
    rewardDelay
  );
  let iterOriginalRoundBlock = nowRoundFirstBlock.toBn();
  while (true) {
    const blockHash = await api.rpc.chain.getBlockHash(iterOriginalRoundBlock);
    const round = await (await api.at(blockHash)).query.parachainStaking.round();
    if (
      round.current.eq(originalRoundNumber) ||
      iterOriginalRoundBlock.sub(round.length).toNumber() < 0
    ) {
      break;
    }

    // go previous round
    iterOriginalRoundBlock = iterOriginalRoundBlock.sub(round.length);
  }
  // we go to the last block of the (original round - 1) since data is snapshotted at round start.
  const originalRoundPriorBlock = iterOriginalRoundBlock.subn(1);
  const originalRoundPriorBlockHash = await api.rpc.chain.getBlockHash(originalRoundPriorBlock);
  const apiAtOriginal = await api.at(originalRoundPriorBlockHash);

  debug(`
  now     ${nowRound.current.toString()} (${nowBlockNumber} / ${nowBlockHash.toHex()})
  round   ${originalRoundNumber.toString()} (prior round last block \
  ${originalRoundPriorBlock} / ${originalRoundPriorBlockHash.toHex()})
  paid in ${nowRoundNumber.toString()} (first block \
  ${nowRoundFirstBlock.toNumber()} / ${nowRoundFirstBlockHash.toHex()} / prior \
  ${priorRewardedBlockHash.toHex()})`);

  // collect info about staked value from collators and delegators
  const apiAtPriorRewarded = await api.at(priorRewardedBlockHash);
  const atStake = await apiAtPriorRewarded.query.parachainStaking.atStake.entries(
    originalRoundNumber
  );
  const stakedValue: StakedValue = {};
  const collatorCount = atStake.length;

  const collators: Set<string> = new Set();
  const delegators: Set<string> = new Set();
  for (const [
    {
      args: [_, accountId],
    },
    { bond, total, delegations },
  ] of atStake) {
    const collatorId = accountId.toHex();
    collators.add(collatorId);
    const points = await apiAtPriorRewarded.query.parachainStaking.awardedPts(
      originalRoundNumber,
      accountId
    );

    const collatorInfo: StakedValueData = {
      id: collatorId,
      bond,
      total,
      points,
      delegators: {},
    };

    const topDelegations = new Set(
      (await apiAtOriginal.query.parachainStaking.topDelegations(accountId))
        .unwrap()
        .delegations.map((d) => d.owner.toHex())
    );
    for (const { owner, amount } of delegations) {
      if (!topDelegations.has(owner.toHex())) {
        continue;
      }
      const id = owner.toHex();
      delegators.add(id);
      collatorInfo.delegators[id] = {
        id: id,
        amount: amount,
      };
    }

    for (const topDelegation of topDelegations) {
      if (!Object.keys(collatorInfo.delegators).includes(topDelegation)) {
        throw new Error(
          `${topDelegation} is missing from collatorInfo ` +
            `for round ${originalRoundNumber.toString()}`
        );
      }
    }
    for (const delegator of Object.keys(collatorInfo.delegators)) {
      if (!topDelegations.has(delegator as any)) {
        throw new Error(
          `${delegator} is missing from topDelegations for round ${originalRoundNumber.toString()}`
        );
      }
    }

    stakedValue[collatorId] = collatorInfo;
  }
  expect(collatorCount).to.equal(
    Object.keys(stakedValue).length,
    `collator count mismatch for round ${originalRoundNumber.toString()}`
  );

  // calculate reward amounts
  const parachainBondInfo = await apiAtPriorRewarded.query.parachainStaking.parachainBondInfo();
  const parachainBondPercent = new Percent(parachainBondInfo.percent);
  const totalStaked = await apiAtPriorRewarded.query.parachainStaking.staked(originalRoundNumber);
  const totalPoints = await apiAtPriorRewarded.query.parachainStaking.points(originalRoundNumber);
  const inflation = await apiAtPriorRewarded.query.parachainStaking.inflationConfig();
  const totalIssuance = await apiAtPriorRewarded.query.balances.totalIssuance();
  const collatorCommissionRate =
    await apiAtPriorRewarded.query.parachainStaking.collatorCommission();

  const range = {
    min: new Perbill(inflation.round.min).of(totalIssuance),
    ideal: new Perbill(inflation.round.ideal).of(totalIssuance),
    max: new Perbill(inflation.round.max).of(totalIssuance),
  };

  const totalRoundIssuance = (function () {
    if (totalStaked.lt(inflation.expect.min)) {
      return range.min;
    } else if (totalStaked.gt(inflation.expect.max)) {
      return range.max;
    } else {
      return range.ideal;
    }
  })();

  // calculate total staking reward
  const firstBlockRewardedEvents = await apiAtRewarded.query.system.events();
  let reservedForParachainBond = new BN(0);
  for (const { phase, event } of firstBlockRewardedEvents) {
    if (!phase.isInitialization) {
      continue;
    }
    // only deduct parachainBondReward if it was transferred (event must exist)
    if (apiAtRewarded.events.parachainStaking.ReservedForParachainBond.is(event)) {
      reservedForParachainBond = event.data[1] as any;
      break;
    }
  }

  const totalStakingReward = (function () {
    const parachainBondReward = parachainBondPercent.of(totalRoundIssuance);
    if (!reservedForParachainBond.isZero()) {
      expect(
        parachainBondReward.eq(reservedForParachainBond),
        `parachain bond amount does not match \
          ${parachainBondReward.toString()} != ${reservedForParachainBond.toString()} \
          for round ${originalRoundNumber.toString()}`
      ).to.be.true;
      return totalRoundIssuance.sub(parachainBondReward);
    }

    return totalRoundIssuance;
  })();

  const delayedPayout = (
    await apiAtRewarded.query.parachainStaking.delayedPayouts(originalRoundNumber)
  ).unwrap();
  expect(
    delayedPayout.totalStakingReward.eq(totalStakingReward),
    `reward amounts do not match \
      ${delayedPayout.totalStakingReward.toString()} != ${totalStakingReward.toString()} \
      for round ${originalRoundNumber.toString()}`
  ).to.be.true;

  // verify rewards
  const latestBlock = await api.rpc.chain.getBlock();
  const latestRoundNumber = latestBlock.block.header.number.toNumber();
  const awardedCollators = (
    await apiAtPriorRewarded.query.parachainStaking.awardedPts.keys(originalRoundNumber)
  ).map((awarded) => awarded.args[1].toHex());

  const awardedCollatorCount = awardedCollators.length;

  const maxRoundChecks = Math.min(latestRoundNumber - nowBlockNumber + 1, awardedCollatorCount);
  debug(`verifying ${maxRoundChecks} blocks for rewards (awarded ${awardedCollatorCount})`);
  const expectedRewardedCollators = new Set(awardedCollators);
  const rewardedCollators = new Set<`0x${string}`>();
  for await (const i of new Array(maxRoundChecks).keys()) {
    const blockNumber = nowRoundFirstBlock.addn(i);
    const rewarded = await assertRewardedEventsAtBlock(
      api,
      blockNumber,
      delegators,
      collators,
      collatorCommissionRate,
      totalRoundIssuance,
      totalPoints,
      totalStakingReward,
      stakedValue
    );

    expect(rewarded.collator, `collator was not rewarded at block ${blockNumber}`).to.exist;

    rewardedCollators.add(rewarded.collator);
    const expectedRewardedDelegators = new Set(
      Object.entries(stakedValue[rewarded.collator].delegators)
        .filter(([_, value]) => !value.amount.isZero())
        .map(([key, _]) => key)
    );

    const notRewarded = new Set(
      [...expectedRewardedDelegators].filter((d) => !rewarded.delegators.has(d))
    );
    const unexpectedlyRewarded = new Set(
      [...rewarded.delegators].filter((d) => !expectedRewardedDelegators.has(d))
    );
    expect(
      notRewarded,
      `delegators "${[...notRewarded].join(", ")}" were not rewarded for collator "${
        rewarded.collator
      }" at block ${blockNumber}`
    ).to.be.empty;
    expect(
      unexpectedlyRewarded,
      `delegators "${[...unexpectedlyRewarded].join(
        ", "
      )}" were unexpectedly rewarded for collator "${rewarded.collator}" at block ${blockNumber}`
    ).to.be.empty;
  }

  const notRewarded = new Set(
    [...expectedRewardedCollators].filter((d) => !rewardedCollators.has(d))
  );
  const unexpectedlyRewarded = new Set(
    [...rewardedCollators].filter((d) => !expectedRewardedCollators.has(d))
  );
  expect(
    unexpectedlyRewarded,
    `collators "${[...unexpectedlyRewarded].join(
      ", "
    )}" were unexpectedly rewarded for round ${originalRoundNumber.toString()}`
  ).to.be.empty;
  expect(
    notRewarded,
    `collators "${[...notRewarded].join(
      ", "
    )}" were not rewarded for round ${originalRoundNumber.toString()}`
  ).to.be.empty;
}

async function assertRewardedEventsAtBlock(
  api: ApiPromise,
  rewardedBlockNumber: BN,
  delegators: Set<string>,
  collators: Set<string>,
  collatorCommissionRate: BN,
  totalRoundIssuance: BN,
  totalPoints: u32,
  totalStakingReward: BN,
  stakedValue: StakedValue
): Promise<Rewarded> {
  const nowRoundRewardBlockHash = await api.rpc.chain.getBlockHash(rewardedBlockNumber);
  const apiAtBlock = await api.at(nowRoundRewardBlockHash);

  debug(`> block ${rewardedBlockNumber} (${nowRoundRewardBlockHash})`);
  const rewards: { [key: `0x${string}`]: { account: string; amount: u128 } } = {};
  const blockEvents = await apiAtBlock.query.system.events();
  let rewardCount = 0;
  for (const { phase, event } of blockEvents) {
    if (!phase.isInitialization) {
      continue;
    }

    if (apiAtBlock.events.parachainStaking.Rewarded.is(event)) {
      rewardCount++;
      rewards[event.data[0].toHex()] = {
        account: event.data[0].toHex(),
        amount: event.data[1] as u128,
      };
    }
  }
  expect(rewardCount).to.equal(Object.keys(rewards).length, "reward count mismatch");

  let delegationReward: BN = new BN(0);
  let collatorInfo: any = {};
  let rewarded = {
    collator: null as `0x${string}`,
    delegators: new Set<string>(),
  };

  for (const accountId of Object.keys(rewards) as `0x${string}`[]) {
    if (collators.has(accountId)) {
      // collator is always paid first so this is guaranteed to execute first
      collatorInfo = stakedValue[accountId];
      const totalCollatorCommissionReward = new Perbill(collatorCommissionRate).of(
        totalRoundIssuance
      );
      const pointsShare = new Perbill(collatorInfo.points, totalPoints);
      const collatorReward = pointsShare.of(totalStakingReward);

      if (!stakedValue[accountId].delegators) {
        assertEqualWithAccount(rewards[accountId].amount, collatorReward, `${accountId} (COL)`);
      } else {
        const collatorCommissionReward = pointsShare.of(totalCollatorCommissionReward);
        delegationReward = collatorReward.sub(collatorCommissionReward);
        const bondShare = new Perbill(collatorInfo.bond, collatorInfo.total);
        const collatorBondReward = bondShare.of(delegationReward);
        const candidateReward = collatorBondReward.add(collatorCommissionReward);
        assertEqualWithAccount(rewards[accountId].amount, candidateReward, `${accountId} (COL)`);
      }
      rewarded.collator = accountId;
    } else if (delegators.has(accountId)) {
      expect(
        collatorInfo.delegators,
        "collator was not paid before the delegator (possibly not at all)"
      ).to.exist;
      // skip checking if rewarded amount was zero
      if (rewards[accountId].amount.isZero()) {
        continue;
      }
      const bondShare = new Perbill(collatorInfo.delegators[accountId].amount, collatorInfo.total);
      const candidateReward = bondShare.of(delegationReward);
      rewarded.delegators.add(accountId);
      assertEqualWithAccount(rewards[accountId].amount, candidateReward, `${accountId} (DEL)`);
    } else {
      throw Error(`invalid key ${accountId}, neither collator not delegator`);
    }
  }

  return rewarded;
}

function assertEqualWithAccount(a: BN, b: BN, account: string) {
  const diff = a.sub(b);

  expect(
    diff.abs().isZero(),
    `${account} ${a.toString()} != ${b.toString()}, difference of ${diff.abs().toString()}`
  ).to.be.true;
}

type Rewarded = { collator: `0x${string}` | null; delegators: Set<string> };

type StakedValueData = {
  id: string;
  bond: u128;
  total: u128;
  points: u32;
  delegators: { [key: string]: { id: string; amount: u128 } };
};

type StakedValue = {
  [key: string]: StakedValueData;
};

class Perthing {
  private unit: BN;
  private perthing: BN;

  constructor(unit: BN, numerator: BN, denominator?: BN) {
    this.unit = unit;
    if (denominator) {
      this.perthing = numerator.mul(unit).div(denominator);
    } else {
      this.perthing = numerator;
    }
  }

  of(value: BN): BN {
    // return this.perthing.mul(value).divRound(this.unit);
    return this.divNearest(this.perthing.mul(value), this.unit);
  }

  toString(): string {
    return `${this.perthing.toString()}`;
  }

  divNearest(a: any, num: BN) {
    var dm = a.divmod(num);

    // Fast case - exact division
    if (dm.mod.isZero()) return dm.div;

    var mod = dm.div.negative !== 0 ? dm.mod.isub(num) : dm.mod;

    var half = num.ushrn(1);
    var r2 = num.andln(1) as any;
    var cmp = mod.cmp(half);

    // Round down
    if (cmp <= 0 || (r2 === 1 && cmp === 0)) return dm.div;

    // Round up
    return dm.div.negative !== 0 ? dm.div.isubn(1) : dm.div.iaddn(1);
  }
}

class Perbill extends Perthing {
  constructor(numerator: BN, denominator?: BN) {
    super(new BN(1_000_000_000), numerator, denominator);
  }
}

class Percent extends Perthing {
  constructor(numerator: BN, denominator?: BN) {
    super(new BN(100), numerator, denominator);
  }
}
