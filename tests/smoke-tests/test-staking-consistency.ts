import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { AccountId20 } from "@polkadot/types/interfaces/runtime";
import { StorageKey, Option } from "@polkadot/types";
import type {
  ParachainStakingDelegator,
  ParachainStakingDelegations,
  ParachainStakingCandidateMetadata,
  ParachainStakingBond,
  ParachainStakingSetOrderedSetBond,
} from "@polkadot/types/lookup";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:staking");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify staking consistency`, { wssUrl, relayWssUrl }, (context) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;
  let maxTopDelegationsPerCandidate: number = 0;
  let allCandidateInfo: [StorageKey<[AccountId20]>, Option<ParachainStakingCandidateMetadata>][];
  let candidatePool: ParachainStakingSetOrderedSetBond;
  let allDelegatorState: [StorageKey<[AccountId20]>, Option<ParachainStakingDelegator>][];
  let allTopDelegations: [StorageKey<[AccountId20]>, Option<ParachainStakingDelegations>][];
  let delegatorsPerCandidates: {
    [index: string]: {
      delegator: string;
      delegation: ParachainStakingBond;
    }[];
  };
  let blocksPerRound: number;
  let minSelectedCandidates: number;
  let totalSelectedCandidates: number;
  let allSelectedCandidates: AccountId20[];

  before("Setup apiAt", async function () {
    // It takes time to load all the accounts.
    this.timeout(180000);

    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
    maxTopDelegationsPerCandidate =
      apiAt.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber();

    allCandidateInfo = await apiAt.query.parachainStaking.candidateInfo.entries();
    allDelegatorState = await apiAt.query.parachainStaking.delegatorState.entries();
    candidatePool = await apiAt.query.parachainStaking.candidatePool();
    allTopDelegations = await apiAt.query.parachainStaking.topDelegations.entries();

    delegatorsPerCandidates = allDelegatorState.reduce(
      (p, state) => {
        for (const delegation of state[1].unwrap().delegations) {
          if (!p[delegation.owner.toHex()]) {
            p[delegation.owner.toHex()] = [];
          }
          p[delegation.owner.toHex()].push({
            delegator: `0x${state[0].toHex().slice(-40)}`,
            delegation,
          });
        }
        return p;
      },
      {} as {
        [key: `0x${string}`]: {
          delegator: `0x${string}`;
          delegation: ParachainStakingBond;
        }[];
      }
    );

    blocksPerRound = (await apiAt.query.parachainStaking.round()).length.toNumber();
    minSelectedCandidates = apiAt.consts.parachainStaking.minSelectedCandidates.toNumber();
    totalSelectedCandidates = (await apiAt.query.parachainStaking.totalSelected()).toNumber();
    allSelectedCandidates = await apiAt.query.parachainStaking.selectedCandidates();
  });

  it("candidate totalCounted matches top X delegations", async function () {
    for (const candidate of allCandidateInfo) {
      const accountId = `0x${candidate[0].toHex().slice(-40)}`;
      const delegators = delegatorsPerCandidates[accountId] || [];

      const expectedTotalCounted =
        delegators
          .map((d) => d.delegation.amount.toBigInt())
          .sort((a, b) => (a < b ? 1 : a > b ? -1 : 0))
          .filter((_, i) => i < maxTopDelegationsPerCandidate)
          .reduce((p, amount) => p + amount, 0n) + candidate[1].unwrap().bond.toBigInt();

      expect(candidate[1].unwrap().totalCounted.toBigInt(), `Candidate: ${accountId}`).to.equal(
        expectedTotalCounted
      );
    }

    debug(
      `Verified ${Object.keys(allCandidateInfo).length} candidates and ${
        allDelegatorState.length
      } delegators`
    );
  });

  it("candidate topDelegator total matches the sum", async function () {
    for (const topDelegation of allTopDelegations) {
      expect(
        topDelegation[1].unwrap().total.toBigInt(),
        `topDelegations of 0x${topDelegation[0].toHex().slice(-40)}`
      ).to.equal(
        topDelegation[1]
          .unwrap()
          .delegations.reduce((p, delegation) => p + delegation.amount.toBigInt(), 0n)
      );
    }
  });

  it("candidate topDelegator total matches candidate totalCounted - bond", async function () {
    for (const candidate of allCandidateInfo) {
      const accountId = `0x${candidate[0].toHex().slice(-40)}`;
      const topDelegation = allTopDelegations
        .find((t) => `0x${t[0].toHex().slice(-40)}` == accountId)[1]
        .unwrap();
      expect(topDelegation.total.toBigInt()).to.equal(
        candidate[1].unwrap().totalCounted.toBigInt() - candidate[1].unwrap().bond.toBigInt()
      );
    }
  });

  it("candidate topDelegations matches top X delegators", async function () {
    for (const candidate of allCandidateInfo) {
      const accountId = `0x${candidate[0].toHex().slice(-40)}`;
      const delegators = delegatorsPerCandidates[accountId] || [];

      const topDelegators = delegators
        .sort((a, b) =>
          a.delegation.amount.toBigInt() < b.delegation.amount.toBigInt()
            ? 1
            : a.delegation.amount.toBigInt() > b.delegation.amount.toBigInt()
            ? -1
            : 0
        )
        .filter((_, i) => i < maxTopDelegationsPerCandidate);

      const topDelegations = allTopDelegations
        .find((t) => `0x${t[0].toHex().slice(-40)}` == accountId)[1]
        .unwrap();

      expect(topDelegations.total.toBigInt()).to.equal(
        topDelegators
          .map((d) => d.delegation.amount.toBigInt())
          .reduce((p, amount) => p + amount, 0n)
      );

      // Verify matching length
      expect(topDelegations.delegations.length).to.equal(topDelegators.length);

      // Verify each delegation amount matches
      // It is not possible to verify the account as there is no deterministic
      // way to differenciate the order of 2 delegators with same amount
      for (const index in topDelegators) {
        expect(
          topDelegators[index].delegation.amount.toBigInt(),
          `topDelegators[${index}] - ${topDelegators[index].delegator}`
        ).to.equal(topDelegations.delegations[index].amount.toBigInt());
      }
    }

    debug(
      `Verified ${Object.keys(allCandidateInfo).length} candidates and ${
        allDelegatorState.length
      } delegators`
    );
  });

  it("all delegators lessTotal matches revoke/decrease requests", async function () {
    let checks = 0;
    if (specVersion >= 1500) {
      const delegationScheduledRequests =
        await apiAt.query.parachainStaking.delegationScheduledRequests.entries();
      const delegatorRequests = delegationScheduledRequests.reduce((p, requests: any) => {
        for (const request of requests[1]) {
          const delegator = request.delegator.toHex();
          if (!p[delegator]) {
            p[delegator] = [];
          }
          p[delegator].push(request);
        }
        return p;
      }, {} as { [delegator: string]: { delegator: any; whenExecutable: any; action: any }[] });

      for (const state of allDelegatorState) {
        const delegator = `0x${state[0].toHex().slice(-40)}`;
        const totalRequestAmount = (delegatorRequests[delegator] || []).reduce(
          (p, v) =>
            p +
            (v.action.isDecrease ? v.action.asDecrease.toBigInt() : v.action.asRevoke.toBigInt()),
          0n
        );

        expect((state[1].unwrap() as any).lessTotal.toBigInt(), `delegator: ${delegator}`).to.equal(
          totalRequestAmount
        );
        checks++;
      }
    }

    if (specVersion < 1500) {
      for (const state of allDelegatorState) {
        const delegator = `0x${state[0].toHex().slice(-40)}`;
        const totalRequestAmount = Array.from(
          (state[1] as any).unwrap().requests.requests.values()
        ).reduce((p, v: any) => p + v.amount.toBigInt(), 0n);

        expect(
          (state[1] as any).unwrap().requests.lessTotal.toBigInt(),
          `delegator: ${delegator}`
        ).to.equal(totalRequestAmount);
        checks++;
      }
    }

    debug(`Verified ${checks} lessTotal (runtime: ${specVersion})`);
  });

  it("candidatePool matches candidateInfo", async function () {
    let foundCandidateInPool = 0;
    for (const candidate of allCandidateInfo) {
      const candidateId = `0x${candidate[0].toHex().slice(-40)}`;
      const candidateData = candidate[1].unwrap();

      if (candidateData.status.isLeaving || candidateData.status.isIdle) {
        expect(
          candidatePool.find((c) => c.owner.toHex() == candidateId),
          `Candidate ${candidateId} is leaving and should not be in the candidate pool`
        ).to.be.undefined;
      } else {
        expect(
          candidatePool.find((c) => c.owner.toHex() == candidateId),
          `Candidate ${candidateId} is active and should be in the candidate pool`
        ).to.not.be.undefined;
        foundCandidateInPool++;
      }
    }

    expect(foundCandidateInPool, "Candidate in pool not matching expected number").to.be.equal(
      candidatePool.length
    );

    debug(
      `Verified ${Object.keys(allCandidateInfo).length} candidates info and ${
        candidatePool.length
      } in the pool`
    );
  });

  it("round length is more than minimum selected candidate count", async function () {
    expect(
      blocksPerRound,
      `blocks per round should be equal or more than the minimum selected candidate count`
    ).to.be.greaterThanOrEqual(minSelectedCandidates);
  });

  it("total selected is more than minimum selected candidate count", async function () {
    expect(
      totalSelectedCandidates,
      `blocks per round should be equal or more than the minimum selected candidate count`
    ).to.be.greaterThanOrEqual(minSelectedCandidates);
  });

  it.skip("current selected candidates are more than minimum required", async function () {
    expect(
      allSelectedCandidates.length,
      `selected candidate count was less than the minimum allowed of ${minSelectedCandidates}`
    ).to.be.greaterThanOrEqual(minSelectedCandidates);
  });

  it("current selected candidates are less than or equal to stored total", async function () {
    expect(
      allSelectedCandidates.length,
      `selected candidate count was less than the minimum allowed of ${minSelectedCandidates}`
    ).to.be.lessThanOrEqual(totalSelectedCandidates);
  });

  it("round length is more than current selected candidates", async function () {
    expect(
      blocksPerRound,
      `blocks per round should be equal or more than the current selected candidates`
    ).to.be.greaterThanOrEqual(allSelectedCandidates.length);
  });
});
