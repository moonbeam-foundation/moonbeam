import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { BN, BN_BILLION } from "@polkadot/util";
import { u128, u32 } from "@polkadot/types";
import { ApiPromise } from "@polkadot/api";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { HexString } from "@polkadot/util/types";
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
  const latestBlock = await api.rpc.chain.getBlock();
  const latestBlockHash = latestBlock.block.hash;
  const latestBlockNumber = latestBlock.block.header.number.toNumber();
  const latestRound = await (await api.at(latestBlock.block.hash)).query.parachainStaking.round();
  const nowBlockHash = await api.rpc.chain.getBlockHash(nowBlockNumber);
  const nowRound = await (await api.at(nowBlockHash)).query.parachainStaking.round();
  const nowRoundNumber = nowRound.current;
  const nowRoundFirstBlock = nowRound.first;
  const nowRoundFirstBlockHash = await api.rpc.chain.getBlockHash(nowRoundFirstBlock);
  const apiAtRewarded = await api.at(nowRoundFirstBlockHash);
  const rewardDelay = apiAtRewarded.consts.parachainStaking.rewardPaymentDelay;
  const priorRewardedBlockHash = await api.rpc.chain.getBlockHash(nowRoundFirstBlock.subn(1));
  const specVersion = (await apiAtRewarded.query.system.lastRuntimeUpgrade())
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
  latest  ${latestRound.current.toString()} (${latestBlockNumber} / ${latestBlockHash.toHex()})
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
    let countedDelegationSum = new BN(0);
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
      countedDelegationSum = countedDelegationSum.add(amount);
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
  const totalCollatorCommissionReward = new Perbill(collatorCommissionRate).of(totalRoundIssuance);

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

  // total expected staking reward minus the amount reserved for parachain bond
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
  const totalBondReward = totalStakingReward.sub(totalCollatorCommissionReward);

  const delayedPayout = (
    await apiAtRewarded.query.parachainStaking.delayedPayouts(originalRoundNumber)
  ).unwrap();
  expect(
    delayedPayout.totalStakingReward.eq(totalStakingReward),
    `reward amounts do not match \
      ${delayedPayout.totalStakingReward.toString()} != ${totalStakingReward.toString()} \
      for round ${originalRoundNumber.toString()}`
  ).to.be.true;

  debug(`totalRoundIssuance            ${totalRoundIssuance.toString()}
reservedForParachainBond      ${reservedForParachainBond} \
(${parachainBondPercent} * totalRoundIssuance)
totalCollatorCommissionReward ${totalCollatorCommissionReward.toString()} \
(${collatorCommissionRate} * totalRoundIssuance)
totalStakingReward            ${totalStakingReward} \
(totalRoundIssuance - reservedForParachainBond)
totalBondReward               ${totalBondReward} \
(totalStakingReward - totalCollatorCommissionReward)`);

  // get the collators to be awarded via `awardedPts` storage
  const awardedCollators = (
    await apiAtPriorRewarded.query.parachainStaking.awardedPts.keys(originalRoundNumber)
  ).map((awarded) => awarded.args[1].toHex());
  const awardedCollatorCount = awardedCollators.length;

  // compute max rounds respecting the current block number and the number of awarded collators
  const maxRoundChecks = Math.min(latestBlockNumber - nowBlockNumber + 1, awardedCollatorCount);
  debug(`verifying ${maxRoundChecks} blocks for rewards (awarded ${awardedCollatorCount})`);
  const expectedRewardedCollators = new Set(awardedCollators);
  const rewardedCollators = new Set<HexString>();
  let totalRewardedAmount = new BN(0);

  // accumulate collator share percentages
  let totalCollatorShare = new BN(0);
  // accumulate amount lost while distributing rewards to delegators per collator
  let totalBondRewardedLoss = new BN(0);
  // accumulate total rewards given to collators & delegators due to bonding
  let totalBondRewarded = new BN(0);
  // accumulate total commission rewards per collator
  let totalCollatorCommissionRewarded = new BN(0);

  // iterate over the next blocks to verify rewards
  for await (const i of new Array(maxRoundChecks).keys()) {
    const blockNumber = nowRoundFirstBlock.addn(i);
    const rewarded = await assertRewardedEventsAtBlock(
      api,
      specVersion,
      blockNumber,
      delegators,
      collators,
      totalCollatorCommissionReward,
      totalPoints,
      totalStakingReward,
      stakedValue
    );
    totalCollatorShare = totalCollatorShare.add(rewarded.collatorSharePerbill);
    totalCollatorCommissionRewarded = totalCollatorCommissionRewarded.add(
      rewarded.amount.commissionReward
    );
    totalRewardedAmount = totalRewardedAmount.add(rewarded.amount.total);
    totalBondRewarded = totalBondRewarded.add(rewarded.amount.bondReward);
    totalBondRewardedLoss = totalBondRewardedLoss.add(rewarded.amount.bondRewardLoss);

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

  // check reward amount with losses due to Perbill arithmetic
  if (specVersion >= 1800) {
    // Perbill arithmetic can deviate at most ±1 per operation so we use the number of collators
    // to compute the max deviation per billion
    const maxDifference = awardedCollatorCount;

    // assert rewarded amounts match (with loss due to Perbill arithmetic)
    const estimatedCommissionRewardedLoss = new Perbill(BN_BILLION.sub(totalCollatorShare)).of(
      totalCollatorCommissionReward
    );
    const actualCommissionRewardedLoss = totalCollatorCommissionReward.sub(
      totalCollatorCommissionRewarded
    );
    const commissionRewardLoss = estimatedCommissionRewardedLoss
      .sub(actualCommissionRewardedLoss)
      .abs();
    expect(
      commissionRewardLoss.lten(maxDifference),
      `Total commission rewarded share loss was above ${maxDifference} parts per billion, \
got "${commissionRewardLoss}", estimated loss ${estimatedCommissionRewardedLoss.toString()}, \
actual loss ${actualCommissionRewardedLoss.toString()}`
    ).to.be.true;

    // we add the two estimated losses, since the totalBondReward is always split between N
    // collators, which then split the reward again between the all the delegators
    const estimatedBondRewardedLoss = new Perbill(BN_BILLION.sub(totalCollatorShare))
      .of(totalBondReward)
      .add(totalBondRewardedLoss);
    const actualBondRewardedLoss = totalBondReward.sub(totalBondRewarded);
    const bondRewardedLoss = estimatedBondRewardedLoss.sub(actualBondRewardedLoss).abs();
    expect(
      bondRewardedLoss.lten(maxDifference),
      `Total bond rewarded share loss was above ${maxDifference} parts per billion, \
got "${bondRewardedLoss}", estimated loss ${estimatedBondRewardedLoss.toString()}, \
actual loss ${actualBondRewardedLoss.toString()}`
    ).to.be.true;

    // calculate total rewarded amount including the amount lost to Perbill arithmetic
    const actualTotalRewardedWithLoss = totalRewardedAmount
      .add(actualCommissionRewardedLoss)
      .add(actualBondRewardedLoss);

    // check that sum of all reward transfers is equal to total expected staking reward
    expect(actualTotalRewardedWithLoss.toString()).to.equal(
      totalStakingReward.toString(),
      `Total rewarded events did not match total expected issuance for collators & delegators, \
      diff of "${actualTotalRewardedWithLoss
        .sub(totalStakingReward)
        .toString()}" for round ${originalRoundNumber}`
    );
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
  specVersion: number,
  rewardedBlockNumber: BN,
  delegators: Set<string>,
  collators: Set<string>,
  totalCollatorCommissionReward: BN,
  totalPoints: u32,
  totalStakingReward: BN,
  stakedValue: StakedValue
): Promise<Rewarded> {
  const nowRoundRewardBlockHash = await api.rpc.chain.getBlockHash(rewardedBlockNumber);
  const apiAtBlock = await api.at(nowRoundRewardBlockHash);

  debug(`> block ${rewardedBlockNumber} (${nowRoundRewardBlockHash})`);
  const rewards: { [key: HexString]: { account: string; amount: u128 } } = {};
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

  let bondReward: BN = new BN(0);
  let collatorInfo: any = {};
  let rewarded = {
    collator: null as HexString,
    delegators: new Set<string>(),
    collatorSharePerbill: new BN(0),
    amount: {
      total: new BN(0),
      commissionReward: new BN(0),
      bondReward: new BN(0),
      bondRewardLoss: new BN(0),
    },
  };
  let totalBondRewardShare = new BN(0);

  for (const accountId of Object.keys(rewards) as HexString[]) {
    rewarded.amount.total = rewarded.amount.total.add(rewards[accountId].amount);

    if (collators.has(accountId)) {
      // collator is always paid first so this is guaranteed to execute first
      collatorInfo = stakedValue[accountId];

      const pointsShare = new Perbill(collatorInfo.points, totalPoints);
      const collatorReward = pointsShare.of(totalStakingReward);
      rewarded.collatorSharePerbill = pointsShare.value();
      const collatorCommissionReward = pointsShare.of(totalCollatorCommissionReward);
      rewarded.amount.commissionReward = collatorCommissionReward;
      bondReward = collatorReward.sub(collatorCommissionReward);

      if (!stakedValue[accountId].delegators) {
        assertEqualWithAccount(rewards[accountId].amount, collatorReward, `${accountId} (COL)`);
      } else {
        const bondShare = new Perbill(collatorInfo.bond, collatorInfo.total);
        totalBondRewardShare = totalBondRewardShare.add(bondShare.value());
        const collatorBondReward = bondShare.of(bondReward);
        rewarded.amount.bondReward = rewarded.amount.bondReward.add(collatorBondReward);
        const collatorTotalReward = collatorBondReward.add(collatorCommissionReward);
        assertEqualWithAccount(
          rewards[accountId].amount,
          collatorTotalReward,
          `${accountId} (COL)`
        );
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
      totalBondRewardShare = totalBondRewardShare.add(bondShare.value());
      const delegatorReward = bondShare.of(bondReward);
      rewarded.amount.bondReward = rewarded.amount.bondReward.add(delegatorReward);
      rewarded.delegators.add(accountId);
      assertEqualWithAccount(rewards[accountId].amount, delegatorReward, `${accountId} (DEL)`);
    } else {
      throw Error(`invalid key ${accountId}, neither collator not delegator`);
    }
  }

  if (specVersion >= 1800) {
    // we calculate the share loss since adding all percentages will usually not yield a full 100%
    const estimatedBondRewardedLoss = new Perbill(BN_BILLION.sub(totalBondRewardShare)).of(
      bondReward
    );
    const actualBondRewardedLoss = bondReward.sub(rewarded.amount.bondReward);

    // Perbill arithmetic can deviate at most ±1 per operation so we use the number of delegators
    // and the collator itself to compute the max deviation per billion
    const maxDifference = rewarded.delegators.size + 1;
    const loss = estimatedBondRewardedLoss.sub(actualBondRewardedLoss).abs();
    expect(
      loss.lten(maxDifference),
      `Total bond rewarded share loss for collator "${rewarded.collator}" was above \
${maxDifference} parts per billion, got diff "${loss}", estimated loss \
${estimatedBondRewardedLoss}, actual loss ${actualBondRewardedLoss}`
    ).to.be.true;

    rewarded.amount.bondRewardLoss = actualBondRewardedLoss;
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

type Rewarded = {
  // Collator account id
  collator: HexString | null;
  // Set of delegator account ids
  delegators: Set<string>;
  // The percentage point share in Perbill of the collator
  collatorSharePerbill: BN;
  // The rewarded amount
  amount: {
    // Total rewarded
    total: BN;
    // Contribution of commission rewards towards the total
    commissionReward: BN;
    // Contribution of bond rewards towards the total
    bondReward: BN;
    // Portion of rewards lost due to Perbill arithmetic (sum of bond shares not 100%)
    bondRewardLoss: BN;
  };
};

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

  value(): BN {
    return this.perthing;
  }

  of(value: BN): BN {
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
