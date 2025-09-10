import "@moonbeam-network/api-augment";
import type { ApiDecoration } from "@polkadot/api/types";
import type { AccountId20 } from "@polkadot/types/interfaces/runtime";
import type { StorageKey, Option } from "@polkadot/types";
import type {
  PalletParachainStakingDelegator,
  PalletParachainStakingDelegations,
  PalletParachainStakingCandidateMetadata,
  PalletParachainStakingBond,
  PalletParachainStakingSetOrderedSet,
} from "@polkadot/types/lookup";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S22",
  title: "Verify staking consistency",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber = 0;
    let apiAt: ApiDecoration<"promise">;
    let specVersion = 0;
    let maxTopDelegationsPerCandidate = 0;
    let allCandidateInfo: [
      StorageKey<[AccountId20]>,
      Option<PalletParachainStakingCandidateMetadata>,
    ][];
    let candidatePool: PalletParachainStakingSetOrderedSet;
    let allDelegatorState: [StorageKey<[AccountId20]>, Option<PalletParachainStakingDelegator>][];
    let allTopDelegations: [StorageKey<[AccountId20]>, Option<PalletParachainStakingDelegations>][];
    let delegatorsPerCandidates: {
      [index: string]: {
        delegator: string;
        delegation: PalletParachainStakingBond;
      }[];
    };
    let blocksPerRound: number;
    let minSelectedCandidates: number;
    let totalSelectedCandidates: number;
    let allSelectedCandidates: AccountId20[];
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
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
            delegation: PalletParachainStakingBond;
          }[];
        }
      );

      blocksPerRound = (await apiAt.query.parachainStaking.round()).length.toNumber();
      minSelectedCandidates = apiAt.consts.parachainStaking.minSelectedCandidates.toNumber();
      totalSelectedCandidates = (await apiAt.query.parachainStaking.totalSelected()).toNumber();
      allSelectedCandidates = await apiAt.query.parachainStaking.selectedCandidates();
    }, 180000);

    it({
      id: "C100",
      title: "candidate totalCounted matches top X delegations",
      test: async function () {
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

        log(
          `Verified ${Object.keys(allCandidateInfo).length} candidates and ${
            allDelegatorState.length
          } delegators`
        );
      },
    });

    it({
      id: "C200",
      title: "candidate topDelegator total matches the sum",
      test: async function () {
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
      },
    });

    it({
      id: "C300",
      title: "candidate topDelegator total matches candidate totalCounted - bond",
      test: async function () {
        for (const candidate of allCandidateInfo) {
          const accountId = `0x${candidate[0].toHex().slice(-40)}`;
          const topDelegation = allTopDelegations
            .find((t) => `0x${t[0].toHex().slice(-40)}` === accountId)![1]
            .unwrap();
          expect(topDelegation.total.toBigInt()).to.equal(
            candidate[1].unwrap().totalCounted.toBigInt() - candidate[1].unwrap().bond.toBigInt()
          );
        }
      },
    });

    it({
      id: "C400",
      title: "candidate topDelegations matches top X delegators",
      test: async function () {
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
            .find((t) => `0x${t[0].toHex().slice(-40)}` === accountId)![1]
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

        log(
          `Verified ${Object.keys(allCandidateInfo).length} candidates and ${
            allDelegatorState.length
          } delegators`
        );
      },
    });

    it({
      id: "C500",
      title: "all delegators lessTotal matches revoke/decrease requests",
      test: async function () {
        let checks = 0;
        if (specVersion >= 1500) {
          const delegationScheduledRequests =
            await apiAt.query.parachainStaking.delegationScheduledRequests.entries();
          const delegatorRequests = delegationScheduledRequests.reduce(
            (p, requests: any) => {
              for (const request of requests[1]) {
                const delegator = request.delegator.toHex();
                if (!p[delegator]) {
                  p[delegator] = [];
                }
                p[delegator].push(request);
              }
              return p;
            },
            {} as { [delegator: string]: { delegator: any; whenExecutable: any; action: any }[] }
          );

          for (const state of allDelegatorState) {
            const delegator = `0x${state[0].toHex().slice(-40)}`;
            const totalRequestAmount = (delegatorRequests[delegator] || []).reduce(
              (p, v) =>
                p +
                (v.action.isDecrease
                  ? v.action.asDecrease.toBigInt()
                  : v.action.asRevoke.toBigInt()),
              0n
            );

            expect(
              (state[1].unwrap() as any).lessTotal.toBigInt(),
              `delegator: ${delegator}`
            ).to.equal(totalRequestAmount);
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

        log(`Verified ${checks} lessTotal (runtime: ${specVersion})`);
      },
    });

    it({
      id: "C600",
      title: "candidatePool matches candidateInfo",
      test: async function () {
        let foundCandidateInPool = 0;
        for (const candidate of allCandidateInfo) {
          const candidateId = `0x${candidate[0].toHex().slice(-40)}`;
          const candidateData = candidate[1].unwrap();

          if (candidateData.status.isLeaving || candidateData.status.isIdle) {
            expect(
              candidatePool.find((c) => c.owner.toHex() === candidateId),
              `Candidate ${candidateId} is leaving and should not be in the candidate pool`
            ).to.be.undefined;
          } else {
            expect(
              candidatePool.find((c) => c.owner.toHex() === candidateId),
              `Candidate ${candidateId} is active and should be in the candidate pool`
            ).to.not.be.undefined;
            foundCandidateInPool++;
          }
        }

        expect(foundCandidateInPool, "Candidate in pool not matching expected number").to.be.equal(
          candidatePool.length
        );

        log(
          `Verified ${Object.keys(allCandidateInfo).length} candidates info and ${
            candidatePool.length
          } in the pool`
        );
      },
    });

    it({
      id: "C700",
      title: "round length is more than minimum selected candidate count",
      test: async function () {
        expect(
          blocksPerRound,
          `blocks per round should be equal or more than the minimum selected candidate count`
        ).to.be.greaterThanOrEqual(minSelectedCandidates);
      },
    });

    it({
      id: "C800",
      title: "total selected is more than minimum selected candidate count",
      test: async function () {
        expect(
          totalSelectedCandidates,
          `blocks per round should be equal or more than the minimum selected candidate count`
        ).to.be.greaterThanOrEqual(minSelectedCandidates);
      },
    });

    it({
      id: "C900",
      title: "current selected candidates are more than minimum required",
      modifier: "skip",
      test: async function () {
        expect(
          allSelectedCandidates.length,
          `selected candidate count was less than the minimum allowed of ${minSelectedCandidates}`
        ).to.be.greaterThanOrEqual(minSelectedCandidates);
      },
    });

    it({
      id: "C1000",
      title: "current selected candidates are less than or equal to stored total",
      test: async function () {
        expect(
          allSelectedCandidates.length,
          `selected candidate count was less than the minimum allowed of ${minSelectedCandidates}`
        ).to.be.lessThanOrEqual(totalSelectedCandidates);
      },
    });

    it({
      id: "C1100",
      title: "round length is more than current selected candidates",
      test: async function () {
        expect(
          blocksPerRound,
          `blocks per round should be equal or more than the current selected candidates`
        ).to.be.greaterThanOrEqual(allSelectedCandidates.length);
      },
    });
  },
});
