import "@moonbeam-network/api-augment";
import { BN, BN_BILLION } from "@polkadot/util";
import { u128, u32, StorageKey, u64 } from "@polkadot/types";
import { ApiPromise } from "@polkadot/api";
import { HexString } from "@polkadot/util/types";
import {
  PalletParachainStakingDelegationRequestsScheduledRequest,
  PalletParachainStakingDelegator,
  PalletParachainStakingCollatorSnapshot,
  PalletParachainStakingBond,
  PalletParachainStakingBondWithAutoCompound,
  PalletParachainStakingRoundInfo,
} from "@polkadot/types/lookup";
import { ApiDecoration } from "@polkadot/api/types";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { FIVE_MINS, ONE_HOURS, Perbill, Percent, TEN_MINS } from "@moonwall/util";
import { rateLimiter, getPreviousRound, getNextRound } from "../../helpers";
import { AccountId20, Block } from "@polkadot/types/interfaces";

const limiter = rateLimiter();

interface RoundData {
  data: PalletParachainStakingRoundInfo;
  firstBlock: Block;
  firstBlockApi: ApiDecoration<"promise">;
  firstBlockSpecVersion: number;
  priorBlock: Block;
  priorBlockApi: ApiDecoration<"promise">;
}

interface PaymentRounds {
  asyncBackingEnabled: boolean;
  firstRewardBlock: Block;
  rewardRound: RoundData; // Stores the last finished round (X)
  roundToPay: RoundData; // Stores the round to pay (X-delay)
  delayedPayoutRound: RoundData; // Stores the round where the delayedPayout is computed (X-delay+1)
}

describeSuite({
  id: "S22",
  title: "When verifying ParachainStaking rewards",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atStakeSnapshot: [StorageKey<[u32, AccountId20]>, PalletParachainStakingCollatorSnapshot][];
    let apiAt: ApiDecoration<"promise">;
    let predecessorApiAt: ApiDecoration<"promise">;
    let paraApi: ApiPromise;

    beforeAll(async () => {
      paraApi = context.polkadotJs("para");

      const atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await paraApi.rpc.chain.getHeader()).number.toNumber();
      const queriedBlockHash = await paraApi.rpc.chain.getBlockHash(atBlockNumber);
      const queryApi = await paraApi.at(queriedBlockHash);
      const queryRound = await queryApi.query.parachainStaking.round();
      log(
        `Querying at block #${queryRound.first.toNumber()}, round #${queryRound.current.toNumber()}`
      );

      const prevBlock = Math.max(queryRound.first.subn(1).toNumber(), 1);
      const prevHash = await paraApi.rpc.chain.getBlockHash(prevBlock);
      apiAt = await paraApi.at(prevHash);
      log(`Snapshot block #${prevBlock} hash ${prevHash.toString()}`);

      const predecessorBlock = (await apiAt.query.parachainStaking.round()).first
        .subn(1)
        .toNumber();
      if (predecessorBlock <= 1) {
        log("Round is too early (fork network probably), skipping test.");
        return; // Replace this with skip() when added to vitest
      }
      const predecessorHash = await paraApi.rpc.chain.getBlockHash(predecessorBlock);
      log(`Reference block #${predecessorBlock} hash ${predecessorHash.toString()}`);
      predecessorApiAt = await paraApi.at(predecessorHash);

      const nowRound = (await apiAt.query.parachainStaking.round()).current.toNumber();
      log(`Loading previous round #${nowRound} snapshot`);
      atStakeSnapshot = await apiAt.query.parachainStaking.atStake.entries(nowRound);
    }, FIVE_MINS);

    it({
      id: "C100",
      title: "should snapshot the selected candidates for that round",
      test: async () => {
        const selectedCandidates = await apiAt.query.parachainStaking.selectedCandidates();
        const totalSelected = (await apiAt.query.parachainStaking.totalSelected()).toNumber();
        expect(atStakeSnapshot.length).to.be.lessThanOrEqual(totalSelected);
        const extras = atStakeSnapshot.filter((item) =>
          selectedCandidates.some((a) => item[0].args[1] == a)
        );
        expect(atStakeSnapshot.length).to.be.equal(selectedCandidates.length);
        expect(extras, `Non-selected candidates in snapshot: ${extras.map((a) => a[0]).join(", ")}`)
          .to.be.empty;
      },
    });

    it({
      id: "C200",
      title: "should have accurate collator stats in snapshot",
      timeout: FIVE_MINS,
      test: async () => {
        const results = await limiter.schedule(() => {
          const specVersion = paraApi.consts.system.version.specVersion.toNumber();
          const allTasks = atStakeSnapshot.map(async (coll, index) => {
            const [
              {
                args: [_, accountId],
              },
              value,
            ] = coll;
            // @ts-expect-error - changed to optional between RT versions
            const { bond, total, delegations } = specVersion < 2600 ? value : value.unwrap();
            const candidateInfo = (
              await limiter.schedule(() =>
                predecessorApiAt.query.parachainStaking.candidateInfo(accountId as AccountId20)
              )
            ).unwrap();

            const bondsMatch: boolean = bond.eq(candidateInfo.bond);
            const delegationsTotalMatch: boolean =
              delegations.length ==
              Math.min(
                candidateInfo.delegationCount.toNumber(),
                predecessorApiAt.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber()
              );
            const totalSum: boolean = delegations
              .reduce((acc: BN, curr) => {
                return acc.add(curr.amount);
              }, new BN(0))
              .add(bond)
              .eq(total);
            return { collator: accountId.toString(), bondsMatch, delegationsTotalMatch, totalSum };
          });

          return Promise.all(allTasks);
        });

        const failures = results.filter((item) => Object.values(item).includes(false));
        expect(
          failures,
          `Checks failed for collators: ${failures.map((a) => a.collator).join(", ")}`
        ).to.be.empty;
      },
    });

    it({
      id: "C300",
      title: "should snapshot candidate delegation amounts correctly",
      timeout: ONE_HOURS,
      test: async () => {
        // This test is slow due to rate limiting, and should be run ad-hoc only
        if (process.env.RUN_ATSTAKE_CONSISTENCY_TESTS != "true") {
          log("Explicit RUN_ATSTAKE_CONSISTENCY_TESTS flag not set to 'true', skipping test");
          return; // Replace this with skip() when added to vitest
        }

        // Function to check a single Delegator's delegation to a collator
        const checkDelegatorDelegation = async (
          accountId: AccountId20,
          delegatorSnapshot: PalletParachainStakingBondWithAutoCompound,
          scheduledRequests: PalletParachainStakingDelegationRequestsScheduledRequest[]
        ) => {
          const { delegations: delegatorDelegations }: PalletParachainStakingDelegator = (
            (await limiter.schedule(() =>
              predecessorApiAt.query.parachainStaking.delegatorState(delegatorSnapshot.owner)
            )) as any
          ).unwrap();

          const delegationAmount = delegatorDelegations.find(
            (candidate) => candidate.owner.toString() == accountId.toString()
          )!.amount;

          // Querying for pending withdrawals which affect the total
          const scheduledRequest = scheduledRequests.find((a) => {
            return a.delegator.toString() == delegatorSnapshot.owner.toString();
          });

          const expected =
            scheduledRequest === undefined
              ? delegationAmount
              : scheduledRequest.action.isDecrease
              ? delegationAmount.sub(scheduledRequest.action.asDecrease)
              : scheduledRequest.action.isRevoke
              ? delegationAmount.sub(scheduledRequest.action.asRevoke)
              : delegationAmount;

          const match = expected.eq(delegatorSnapshot.amount);
          if (!match) {
            log(
              "Snapshot amount " +
                delegatorSnapshot.amount.toString() +
                " does not match storage amount " +
                delegationAmount.toString() +
                " for delegator: " +
                delegatorSnapshot.owner.toString() +
                " on candidate: " +
                accountId.toString()
            );
          }
          return {
            collator: accountId.toString(),
            delegator: delegatorSnapshot.owner.toString(),
            match,
          };
        };
        log(`Gathering snapshot query requests for ${atStakeSnapshot.length} collators.`);
        const promises = atStakeSnapshot.map(async (coll) => {
          const [
            {
              args: [_, accountId],
            },
            { bond, total, delegations },
          ] = coll;
          const scheduledRequests = await limiter.schedule(() =>
            predecessorApiAt.query.parachainStaking.delegationScheduledRequests(
              accountId as AccountId20
            )
          );

          return Promise.all(
            delegations.map((delegation) =>
              checkDelegatorDelegation(accountId, delegation, scheduledRequests)
            )
          );
        });

        // RPC endpoints roughly rate limit to 10 queries a second
        const delegationCount = atStakeSnapshot
          .map(([_, { delegations }]) => delegations.length)
          .reduce((acc, curr) => acc + curr, 0);
        const estimatedTime = ((delegationCount + atStakeSnapshot.length) / 600).toFixed(2);
        log(
          "With a count of " +
            delegationCount +
            " delegations, this may take upto " +
            estimatedTime +
            " mins."
        );

        const results = await Promise.all(promises);
        const mismatches = results.flatMap((a) => a).filter((item) => item.match == false);
        expect(
          mismatches,
          `Mismatched amounts for ${mismatches
            .map((a) => `delegator ${a.delegator} collator:${a.collator}`)
            .join(", ")}`
        ).to.be.empty;

        // Exit buffer for cleanup
        await new Promise((resolve) => setTimeout(resolve, 2000));
      },
    });

    it({
      id: "C400",
      title: "should snapshot delegate autocompound preferences correctly",
      timeout: ONE_HOURS,
      test: async () => {
        if (process.env.RUN_ATSTAKE_CONSISTENCY_TESTS != "true") {
          log("Explicit RUN_ATSTAKE_CONSISTENCY_TESTS flag not set to 'true', skipping test");
          return; // Replace this with skip() when added to vitest
        }
        const specVersion = paraApi.consts.system.version.specVersion.toNumber();
        if (specVersion < 1900) {
          log(`Autocompounding not supported for ${specVersion}, skipping test.`);
          return; // Replace this with skip() when added to vitest
        }

        // Function to check a single Delegator's delegation to a collator
        const checkDelegatorAutocompound = async (
          collatorId: AccountId20,
          delegatorSnapshot: PalletParachainStakingBond | any,
          autoCompoundPrefs: any[]
        ) => {
          const autoCompoundQuery = autoCompoundPrefs.find(
            (a) => a.delegator.toString() == delegatorSnapshot.owner.toString()
          );
          const autoCompoundAmount =
            autoCompoundQuery == undefined ? new BN(0) : autoCompoundQuery.value;
          const match = autoCompoundAmount.eq(delegatorSnapshot.autoCompound);
          if (!match) {
            log(
              "Snapshot autocompound " +
                delegatorSnapshot.autoCompound.toString() +
                "% does not match storage autocompound " +
                autoCompoundAmount.toString() +
                "% for delegator: " +
                delegatorSnapshot.owner.toString() +
                " on candidate: " +
                collatorId.toString()
            );
          }
          return {
            collator: collatorId.toString(),
            delegator: delegatorSnapshot.owner.toString(),
            match,
          };
        };

        log(`Gathering snapshot query requests for ${atStakeSnapshot.length} collators.`);
        const promises = atStakeSnapshot
          .map(
            async ([
              {
                args: [_, accountId],
              },
              { delegations },
            ]) => {
              const autoCompoundPrefs = (await limiter.schedule(() =>
                predecessorApiAt.query.parachainStaking.autoCompoundingDelegations(accountId)
              )) as any;

              return delegations.map((delegation) =>
                checkDelegatorAutocompound(accountId, delegation, autoCompoundPrefs)
              );
            }
          )
          .flatMap((a) => a);

        // RPC endpoints roughly rate limit to 10 queries a second
        const estimatedTime = (promises.length / 600).toFixed(2);
        log("Verifying autoCompound preferences, estimated time " + estimatedTime + " mins.");

        const results: any = await Promise.all(promises);
        const mismatches = results.filter((item: any) => item.match == false);
        expect(
          mismatches,
          `Mismatched autoCompound for ${mismatches
            .map((a: any) => `delegator ${a.delegator} collator:${a.collator}`)
            .join(", ")}`
        ).to.be.empty;

        // Exit buffer for cleanup
        await new Promise((resolve) => setTimeout(resolve, 2000));
      },
    });

    it({
      id: "C500",
      title: "rewards are given as expected",
      timeout: TEN_MINS,
      test: async () => {
        const atBlockNumber = process.env.BLOCK_NUMBER
          ? parseInt(process.env.BLOCK_NUMBER)
          : (await paraApi.rpc.chain.getHeader()).number.toNumber();

        await assertRewardsAtRoundBefore(paraApi, atBlockNumber);
      },
    });

    const assertRewardsAtRoundBefore = async (api: ApiPromise, nowBlockNumber: number) => {
      const nowBlockHash = await api.rpc.chain.getBlockHash(nowBlockNumber);
      const nowRound = await (await api.at(nowBlockHash)).query.parachainStaking.round();
      const previousRoundBlock = nowRound.first.subn(1).toNumber();

      await assertRewardsAt(api, previousRoundBlock);
    };

    const assertRewardsAt = async (api: ApiPromise, nowBlockNumber: number) => {
      const latestBlock = await api.rpc.chain.getBlock();
      const latestBlockHash = latestBlock.block.hash;
      const latestBlockNumber = latestBlock.block.header.number.toNumber();
      const latestRound = await (
        await api.at(latestBlock.block.hash)
      ).query.parachainStaking.round();

      // Loads information and APIs for a given round
      const loadRoundData = async (blockNumber: BN): Promise<RoundData> => {
        console.log(`Loading round data for block number ${blockNumber}`);
        let firstBlockNumber: BN;
        let blockHash = await api.rpc.chain.getBlockHash(blockNumber);
        let apiAt = await api.at(blockHash);
        const data = await apiAt.query.parachainStaking.round();
        // update the block number and api with the right block
        if (!data.first.toBn().eq(blockNumber)) {
          firstBlockNumber = data.first.toBn();
          blockHash = await api.rpc.chain.getBlockHash(data.first);
          apiAt = await api.at(blockHash);
        } else {
          firstBlockNumber = blockNumber;
        }

        const priorBlockHash = await api.rpc.chain.getBlockHash(firstBlockNumber.subn(1));
        const priorApiAt = await api.at(priorBlockHash);

        return {
          data,
          firstBlock: (await api.rpc.chain.getBlock(blockHash)).block,
          firstBlockApi: apiAt,
          firstBlockSpecVersion: (await apiAt.query.system.lastRuntimeUpgrade())
            .unwrap()
            .specVersion.toNumber(),
          priorBlock: (await api.rpc.chain.getBlock(priorBlockHash)).block,
          priorBlockApi: priorApiAt,
        };
      };

      // Loads all information related to rounds involved in the reward
      const loadPaymentRounds = async (blockNumber: BN): Promise<PaymentRounds> => {
        const rewardRound = await loadRoundData(blockNumber);

        const rewardDelay = rewardRound.firstBlockApi.consts.parachainStaking.rewardPaymentDelay;

        const roundToPay = await loadRoundData(
          (await getPreviousRound(api, rewardRound.data, rewardDelay)).first.toBn()
        );
        const delayedPayoutRound = await loadRoundData(
          (await getNextRound(api, roundToPay.data)).first.toBn()
        );

        // Payment of collators have been moved 1 block later at runtime 2100
        const delayBlockReward = rewardRound.firstBlockSpecVersion >= 2100 ? new BN(1) : new BN(0);

        const firstRewardHash = await api.rpc.chain.getBlockHash(
          rewardRound.firstBlock.header.number.toBn().add(delayBlockReward)
        );
        const firstRewardBlock = (await api.rpc.chain.getBlock(firstRewardHash)).block;

        const asyncBackingEnabled = !!api.runtimeMetadata.asLatest.pallets.find(
          ({ name }) => name.toHuman() === "AsyncBacking"
        );

        const payment: PaymentRounds = {
          asyncBackingEnabled,
          firstRewardBlock,
          rewardRound,
          roundToPay,
          delayedPayoutRound,
        };

        return payment;
      };

      const payment: PaymentRounds = await loadPaymentRounds(new BN(nowBlockNumber));

      log(
        `
      latest  ${latestRound.current.toString()} ` +
          `(${latestBlockNumber} / ${latestBlockHash.toHex()})
      rewarded round ${payment.rewardRound.data.current.toString()} - ` +
          `spec: ${payment.rewardRound.firstBlockSpecVersion.toString()}
        reward: #${payment.firstRewardBlock.header.number.toString()} / ` +
          `[${payment.firstRewardBlock.header.hash.toHex()}]
         first: #${payment.rewardRound.firstBlock.header.number.toString()} / ` +
          `[${payment.rewardRound.firstBlock.header.hash.toHex()}]
         prior: #${payment.rewardRound.priorBlock.header.number.toString()} / ` +
          `[${payment.rewardRound.priorBlock.header.hash.toHex()}]
      delayed payout computation round ${payment.delayedPayoutRound.data.current.toString()} - ` +
          `spec: ${payment.rewardRound.firstBlockSpecVersion.toString()}
         first: #${payment.delayedPayoutRound.firstBlock.header.number.toString()} / ` +
          `[${payment.delayedPayoutRound.firstBlock.header.hash.toHex()}]
         prior: #${payment.delayedPayoutRound.priorBlock.header.number.toString()} / ` +
          `[${payment.delayedPayoutRound.priorBlock.header.hash.toHex()}]
      round to pay ${payment.roundToPay.data.current.toString()} - ` +
          `spec: ${payment.rewardRound.firstBlockSpecVersion.toString()}
         first: #${payment.roundToPay.firstBlock.header.number.toString()} / ` +
          `[${payment.roundToPay.firstBlock.header.hash.toHex()}]
         prior: #${payment.roundToPay.priorBlock.header.number.toString()} / ` +
          `[${payment.roundToPay.priorBlock.header.hash.toHex()}]`
      );

      // collect info about staked value from collators and delegators
      const atStake =
        await payment.rewardRound.priorBlockApi.query.parachainStaking.atStake.entries(
          payment.roundToPay.data.current
        );
      const stakedValue: StakedValue = {};
      const collatorCount = atStake.length;

      const collators: Set<string> = new Set();
      const delegators: Set<string> = new Set();
      for (const [
        {
          args: [_, accountId],
        },
        value,
      ] of atStake) {
        // @ts-expect-error - changed to optional between RT versions
        const { bond, total, delegations } =
          payment.rewardRound.firstBlockSpecVersion < 2600 ? value : value.unwrap();
        const collatorId = accountId.toHex();
        collators.add(collatorId);
        const points = await payment.rewardRound.priorBlockApi.query.parachainStaking.awardedPts(
          payment.roundToPay.data.current,
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
          (await payment.roundToPay.firstBlockApi.query.parachainStaking.topDelegations(accountId))
            .unwrap()
            .delegations.map((d) => d.owner.toHex())
        );
        let countedDelegationSum = new BN(0);
        if (payment.roundToPay.firstBlockSpecVersion >= 1900) {
          for (const { owner, amount, autoCompound } of delegations as any) {
            if (!topDelegations.has(owner.toHex())) {
              continue;
            }
            const id = owner.toHex();
            delegators.add(id);
            collatorInfo.delegators[id] = {
              id: id,
              amount: amount,
              autoCompound: new Percent(autoCompound.toNumber()),
            };
            countedDelegationSum = countedDelegationSum.add(amount);
          }
        } else {
          for (const { owner, amount } of delegations) {
            if (!topDelegations.has(owner.toHex())) {
              continue;
            }
            const id = owner.toHex();
            delegators.add(id);
            collatorInfo.delegators[id] = {
              id: id,
              amount: amount,
              autoCompound: new Percent(0),
            };
            countedDelegationSum = countedDelegationSum.add(amount);
          }
        }

        for (const topDelegation of topDelegations) {
          if (!Object.keys(collatorInfo.delegators).includes(topDelegation)) {
            throw new Error(
              `${topDelegation} is missing from collatorInfo ` +
                `for round ${payment.roundToPay.data.current.toString()}`
            );
          }
        }
        for (const delegator of Object.keys(collatorInfo.delegators)) {
          if (!topDelegations.has(delegator as any)) {
            throw new Error(
              `${delegator} is missing from topDelegations for round` +
                ` ${payment.roundToPay.data.current.toString()}`
            );
          }
        }

        stakedValue[collatorId] = collatorInfo;
      }
      expect(collatorCount).to.equal(
        Object.keys(stakedValue).length,
        `collator count mismatch for round ${payment.roundToPay.data.current.toString()}`
      );

      // calculate reward amounts
      let totalRoundIssuance: BN;
      if (payment.asyncBackingEnabled) {
        // Formula:
        //   totalRoundIssuance = (roundDuration / idealDuration) * idealIssuance

        const currentSlot: u64 = (
          await payment.delayedPayoutRound.priorBlockApi.query.asyncBacking.slotInfo()
        ).unwrap()[0];

        const firstSlot = (
          await payment.delayedPayoutRound.priorBlockApi.query.parachainStaking.round()
        ).firstSlot;
        const slotDuration =
          payment.delayedPayoutRound.priorBlockApi.consts.parachainStaking.slotDuration;
        const roundDuration = currentSlot.sub(firstSlot).mul(slotDuration);
        const idealDuration = (
          await payment.delayedPayoutRound.priorBlockApi.query.parachainStaking.round()
        ).length.mul(payment.delayedPayoutRound.priorBlockApi.consts.parachainStaking.blockTime);

        const totalIssuance =
          await payment.delayedPayoutRound.priorBlockApi.query.balances.totalIssuance();
        const idealInflation = (
          await payment.delayedPayoutRound.priorBlockApi.query.parachainStaking.inflationConfig()
        ).round.ideal;
        const idealIssuance = new Perbill(idealInflation).of(totalIssuance);

        totalRoundIssuance = roundDuration.mul(idealIssuance).div(idealDuration);
      } else {
        // Always apply max inflation
        // It works because the total staked amount is already 1000 times more than the max on
        // production, so it's very unlikely to change before RT2801 deployment on moonbeam

        const totalIssuance =
          await payment.delayedPayoutRound.priorBlockApi.query.balances.totalIssuance();

        const inflation =
          await payment.delayedPayoutRound.priorBlockApi.query.parachainStaking.inflationConfig();
        const range = {
          min: new Perbill(inflation.round.min).of(totalIssuance),
          ideal: new Perbill(inflation.round.ideal).of(totalIssuance),
          max: new Perbill(inflation.round.max).of(totalIssuance),
        };

        totalRoundIssuance = range.max;
      }

      const collatorCommissionRate =
        await payment.rewardRound.priorBlockApi.query.parachainStaking.collatorCommission();

      const totalCollatorCommissionReward = new Perbill(collatorCommissionRate).of(
        totalRoundIssuance
      );

      // calculate total staking reward
      const firstBlockRewardedEvents =
        await payment.delayedPayoutRound.firstBlockApi.query.system.events();
      let reservedForParachainBond = new BN(0);
      for (const { phase, event } of firstBlockRewardedEvents) {
        if (!phase.isInitialization) {
          continue;
        }
        const eventTypes = payment.delayedPayoutRound.firstBlockApi.events;
        // only deduct parachainBondReward if it was transferred (event must exist)
        if (eventTypes.parachainStaking.ReservedForParachainBond.is(event)) {
          reservedForParachainBond = event.data[1] as any;
          break;
        }
      }

      const parachainBondInfo = (
        await payment.rewardRound.priorBlockApi.query.parachainStaking.inflationDistributionInfo(
          "ParachainBondReserve"
        )
      ).value;
      const totalPoints = await payment.rewardRound.priorBlockApi.query.parachainStaking.points(
        payment.roundToPay.data.current
      );
      const parachainBondPercent = new Percent(parachainBondInfo.percent);
      // total expected staking reward minus the amount reserved for parachain bond
      const totalStakingReward = (() => {
        const parachainBondReward = parachainBondPercent.of(totalRoundIssuance);
        if (!reservedForParachainBond.isZero()) {
          expect(
            parachainBondReward.eq(reservedForParachainBond),
            `parachain bond amount does not match \
              ${parachainBondReward.toString()} != ${reservedForParachainBond.toString()} \
              for round ${payment.roundToPay.data.current.toString()}`
          ).to.be.true;
          return totalRoundIssuance.sub(parachainBondReward);
        }

        return totalRoundIssuance;
      })();
      const totalBondReward = totalStakingReward.sub(totalCollatorCommissionReward);

      log(`
    paidRoundNumber               ${payment.roundToPay.data.current.toString()}
    totalRoundIssuance            ${totalRoundIssuance.toString()}
    reservedForParachainBond      ${reservedForParachainBond} \
    (${parachainBondPercent} * totalRoundIssuance)
    totalCollatorCommissionReward ${totalCollatorCommissionReward.toString()} \
    (${collatorCommissionRate} * totalRoundIssuance)
    totalStakingReward            ${totalStakingReward} \
    (totalRoundIssuance - reservedForParachainBond)
    totalBondReward               ${totalBondReward} \
    (totalStakingReward - totalCollatorCommissionReward)`);

      const delayedPayout = (
        await payment.rewardRound.firstBlockApi.query.parachainStaking.delayedPayouts(
          payment.roundToPay.data.current
        )
      ).unwrap();
      expect(
        delayedPayout.totalStakingReward.eq(totalStakingReward),
        `reward amounts do not match \
        ${delayedPayout.totalStakingReward.toString()} != ${totalStakingReward.toString()} \
        for round ${payment.roundToPay.data.current.toString()}`
      ).to.be.true;

      // get the collators to be awarded via `awardedPts` storage
      const awardedCollators = (
        await payment.rewardRound.priorBlockApi.query.parachainStaking.awardedPts.keys(
          payment.roundToPay.data.current
        )
      ).map((awarded) => awarded.args[1].toHex());
      const awardedCollatorCount = awardedCollators.length;

      // compute max rounds respecting the current block number and the number of awarded collators
      const maxRewardedBlocks = Math.min(
        latestBlockNumber - payment.rewardRound.firstBlock.header.number.toNumber() + 1,
        collatorCount
      );
      log(`verifying ${maxRewardedBlocks} blocks for rewards (awarded ${awardedCollatorCount})`);
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

      let skippedRewardEvents = 0;
      // iterate over the next blocks to verify rewards
      for await (const i of new Array(maxRewardedBlocks).keys()) {
        const blockNumber = payment.firstRewardBlock.header.number.toBn().addn(i);
        const blockHash = await api.rpc.chain.getBlockHash(blockNumber);
        const apiAtBlock = await api.at(blockHash);

        const outstandingRevokesAtBlock: { [key: string]: Set<string> } = (
          await apiAtBlock.query.parachainStaking.delegationScheduledRequests.entries()
        ).reduce(
          (
            acc,
            [
              {
                args: [candidateId],
              },
              scheduledRequests,
            ]
          ) => {
            if (!(candidateId.toHex() in acc)) {
              acc[candidateId.toHex()] = new Set();
            }
            scheduledRequests
              .filter((req) => req.action.isRevoke)
              .forEach((req) => {
                acc[candidateId.toHex()].add(req.delegator.toHex());
              });
            return acc;
          },
          {} as { [key: string]: Set<string> }
        );

        const { rewarded, autoCompounded } = await assertRewardedEventsAtBlock(
          api,
          payment.rewardRound.firstBlockSpecVersion,
          blockNumber,
          delegators,
          collators,
          totalCollatorCommissionReward,
          totalPoints,
          totalStakingReward,
          stakedValue,
          outstandingRevokesAtBlock
        );
        totalCollatorShare = totalCollatorShare.add(rewarded.collatorSharePerbill);
        totalCollatorCommissionRewarded = totalCollatorCommissionRewarded.add(
          rewarded.amount.commissionReward
        );
        totalRewardedAmount = totalRewardedAmount.add(rewarded.amount.total);
        totalBondRewarded = totalBondRewarded.add(rewarded.amount.bondReward);
        totalBondRewardedLoss = totalBondRewardedLoss.add(rewarded.amount.bondRewardLoss);

        // This occurs when a collator did not produce any blocks, when rewards were being paid out.
        // Since collators are fetched from AtStake, a collator that is not producing blocks will
        // still be checked for rewards, but not be paid.
        if (!rewarded.collator) {
          log(`no collator was not rewarded at block ${blockNumber}`);
          skippedRewardEvents += 1;
          continue;
        }
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
          )}" were unexpectedly rewarded for collator "${
            rewarded.collator
          }" at block ${blockNumber}`
        ).to.be.empty;

        if (payment.rewardRound.firstBlockSpecVersion >= 1900) {
          const expectedAutoCompoundedDelegators = new Set(
            Object.entries(stakedValue[rewarded.collator].delegators)
              .filter(
                ([key, { autoCompound }]) =>
                  !autoCompound.value().isZero() &&
                  expectedRewardedDelegators.has(key) &&
                  !outstandingRevokesAtBlock[rewarded.collator!]?.has(key)
              )
              .map(([key, _]) => key)
          );
          const notAutoCompounded = new Set(
            [...expectedAutoCompoundedDelegators].filter((d) => !autoCompounded.has(d))
          );
          const unexpectedlyAutoCompounded = new Set(
            [...autoCompounded].filter((d) => !expectedAutoCompoundedDelegators.has(d))
          );
          expect(
            notAutoCompounded,
            `delegators "${[...notAutoCompounded].join(
              ", "
            )}" were not auto-compounded for collator "${
              rewarded.collator
            }" at block ${blockNumber}`
          ).to.be.empty;
          expect(
            unexpectedlyAutoCompounded,
            `delegators "${[...unexpectedlyAutoCompounded].join(
              ", "
            )}" were unexpectedly auto-compounded for collator "${
              rewarded.collator
            }" at block ${blockNumber}`
          ).to.be.empty;
        }
      }

      // check reward amount with losses due to Perbill arithmetic
      if (payment.rewardRound.firstBlockSpecVersion >= 1800) {
        // Perbill arithmetic can deviate at most ±1 per operation so we use the number of
        // collators to compute the max deviation per billion
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
            .toString()}" for round ${payment.roundToPay.data.current.toString()}`
        );
      }

      expect(skippedRewardEvents).to.be.eq(collatorCount - rewardedCollators.size);
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
        )}" were unexpectedly rewarded for round ${payment.roundToPay.data.current.toString()}`
      ).to.be.empty;
      expect(
        notRewarded,
        `collators "${[...notRewarded].join(
          ", "
        )}" were not rewarded for round ${payment.roundToPay.data.current.toString()}`
      ).to.be.empty;
    };

    interface Rewards {
      [key: HexString]: { account: string; amount: u128 };
    }

    const assertRewardedEventsAtBlock = async (
      api: ApiPromise,
      specVersion: number,
      rewardedBlockNumber: BN,
      delegators: Set<string>,
      collators: Set<string>,
      totalCollatorCommissionReward: BN,
      totalPoints: u32,
      totalStakingReward: BN,
      stakedValue: StakedValue,
      outstandingRevokes: { [key: string]: Set<string> }
    ): Promise<{ rewarded: Rewarded; autoCompounded: Set<string> }> => {
      const nowRoundRewardBlockHash = await api.rpc.chain.getBlockHash(rewardedBlockNumber);
      const apiAtBlock = await api.at(nowRoundRewardBlockHash);
      const apiAtPreviousBlock = await api.at(
        await api.rpc.chain.getBlockHash(rewardedBlockNumber.toNumber() - 1)
      );

      const round = await apiAtBlock.query.parachainStaking.round();

      log(`> block ${rewardedBlockNumber} (${nowRoundRewardBlockHash})`);
      const rewards: Rewards = {};
      const autoCompounds: {
        [key: HexString]: { candidate: string; account: string; amount: u128 };
      } = {};
      const blockEvents = await apiAtBlock.query.system.events();
      let rewardCount = 0;
      let autoCompoundCount = 0;
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

        if (specVersion >= 2000) {
          // Now orbiters have their own event. To replicate previous behavior,
          // we take the collator associated and mark rewards as if they were
          // to the collator
          if (apiAtBlock.events.moonbeamOrbiters.OrbiterRewarded.is(event)) {
            rewardCount++;
            // The orbiter is removed from the list at the block of the reward so we query the
            // previous block instead.
            // The round rewarded is 2 rounds before the current one.
            const collators =
              await apiAtPreviousBlock.query.moonbeamOrbiters.orbiterPerRound.entries(
                round.current.toNumber() - 2
              );

            const collator = `0x${collators
              .find((orbit) => orbit[1].toHex() == event.data[0].toHex())![0]
              .toHex()
              .slice(-40)}`;
            rewards[collator as HexString] = {
              account: collator,
              amount: event.data[1] as u128,
            };
          }
        }

        if (specVersion >= 1900) {
          if (apiAtBlock.events.parachainStaking.Compounded.is(event)) {
            autoCompoundCount++;
            autoCompounds[event.data[1].toHex()] = {
              candidate: event.data[0].toHex(),
              account: event.data[1].toHex(),
              amount: event.data[2] as u128,
            };
          }
        }
      }
      expect(rewardCount).to.equal(Object.keys(rewards).length, "reward count mismatch");
      expect(autoCompoundCount).to.equal(
        Object.keys(autoCompounds).length,
        "autoCompound count mismatch"
      );

      let bondReward: BN = new BN(0);
      let collatorInfo: any = {};
      const rewarded: Rewarded = {
        delegators: new Set<string>(),
        collatorSharePerbill: new BN(0),
        amount: {
          total: new BN(0),
          commissionReward: new BN(0),
          bondReward: new BN(0),
          bondRewardLoss: new BN(0),
        },
      };
      const autoCompounded = new Set<string>();
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
            expect(rewards[accountId].amount.eq(collatorReward), `${accountId} (COL) - Reward`).to
              .be.true;
          } else {
            const bondShare = new Perbill(collatorInfo.bond, collatorInfo.total);
            totalBondRewardShare = totalBondRewardShare.add(bondShare.value());
            const collatorBondReward = bondShare.of(bondReward);
            rewarded.amount.bondReward = rewarded.amount.bondReward.add(collatorBondReward);
            const collatorTotalReward = collatorBondReward.add(collatorCommissionReward);

            expect(rewards[accountId].amount.eq(collatorTotalReward), `${accountId} (COL) - Reward`)
              .to.be.true;
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

          // check reward
          const bondShare = new Perbill(
            collatorInfo.delegators[accountId].amount,
            collatorInfo.total
          );
          totalBondRewardShare = totalBondRewardShare.add(bondShare.value());
          const delegatorReward = bondShare.of(bondReward);
          rewarded.amount.bondReward = rewarded.amount.bondReward.add(delegatorReward);
          rewarded.delegators.add(accountId);
          expect(rewards[accountId].amount.eq(delegatorReward), `${accountId} (DEL) - Reward`).to.be
            .true;

          const canAutoCompound =
            !outstandingRevokes[rewarded.collator!] ||
            !outstandingRevokes[rewarded.collator!].has(accountId);
          if (specVersion >= 1900 && canAutoCompound) {
            const autoCompoundPercent = collatorInfo.delegators[accountId].autoCompound;
            // skip assertion if auto-compound 0%
            if (autoCompoundPercent.value().isZero()) {
              continue;
            }
            const autoCompoundReward = autoCompoundPercent.ofCeil(rewards[accountId].amount);
            if (autoCompounds[accountId]) {
              expect(
                autoCompounds[accountId].amount.eq(autoCompoundReward),
                `${accountId} (DEL) - AutoCompound ${autoCompoundPercent.toString()}% of ${rewards[
                  accountId
                ].amount.toString()}`
              ).to.be.true;
              autoCompounded.add(accountId);
            }
          }
        } else {
          throw Error(`invalid key ${accountId}, neither collator not delegator`);
        }
      }

      // return if no one was rewarded this round
      if (!rewarded.collator) {
        return { rewarded, autoCompounded };
      }

      if (specVersion >= 1800) {
        // we calculate the share loss since adding all percentages will usually not yield a 100%
        const estimatedBondRewardedLoss = new Perbill(BN_BILLION.sub(totalBondRewardShare)).of(
          bondReward
        );
        const actualBondRewardedLoss = bondReward.sub(rewarded.amount.bondReward);

        // Perbill arithmetic can deviate at most ±1 per operation so we use the number of
        // delegators and the collator itself to compute the max deviation per billion
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

      return { rewarded, autoCompounded };
    };

    type Rewarded = {
      // Collator account id
      collator?: HexString;
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
      delegators: { [key: string]: { id: string; amount: u128; autoCompound: Percent } };
    };

    type StakedValue = {
      [key: string]: StakedValueData;
    };
  },
});
