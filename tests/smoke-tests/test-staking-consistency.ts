import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { AccountId20 } from "@polkadot/types/interfaces/runtime";
import { StorageKey, Option } from "@polkadot/types";
import type {
  FrameSystemAccountInfo,
  ParachainStakingDelegator,
  ParachainStakingDelegations,
  ParachainStakingCandidateMetadata,
  ParachainStakingBond,
} from "@polkadot/types/lookup";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:staking");

const wssUrl = process.env.WSS_URL || null;

describeSmokeSuite(`Verify staking consistency`, { wssUrl }, (context) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;
  let maxTopDelegationsPerCandidate: number = 0;
  let allCandidateInfo: [StorageKey<[AccountId20]>, Option<ParachainStakingCandidateMetadata>][];
  let allDelegatorState: [StorageKey<[AccountId20]>, Option<ParachainStakingDelegator>][];
  let allTopDelegations: [StorageKey<[AccountId20]>, Option<ParachainStakingDelegations>][];
  let delegatorsPerCandidates: {
    [index: string]: {
      delegator: string;
      delegation: ParachainStakingBond;
    }[];
  };

  before("Setup apiAt", async function () {
    // It takes time to load all the accounts.
    this.timeout(120000);

    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
    maxTopDelegationsPerCandidate =
      apiAt.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber();

    allCandidateInfo = await apiAt.query.parachainStaking.candidateInfo.entries();
    allDelegatorState = await apiAt.query.parachainStaking.delegatorState.entries();
    allTopDelegations = await apiAt.query.parachainStaking.topDelegations.entries();

    delegatorsPerCandidates = allDelegatorState.reduce((p, state) => {
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
    }, {});
  });

  it("candidate totalCounted matches top X delegations", async function () {
    this.timeout(120000);
    // Load data

    for (const candidate of allCandidateInfo) {
      const accountId = `0x${candidate[0].toHex().slice(-40)}`;
      const delegators = delegatorsPerCandidates[accountId] || [];

      const expectedTotalCounted =
        delegators
          .map((d) => d.delegation.amount.toBigInt())
          .sort((a, b) => (a < b ? 1 : a > b ? -1 : 0))
          .filter((_, i) => i < maxTopDelegationsPerCandidate)
          .reduce((p, amount) => p + amount, 0n) + candidate[1].unwrap().bond.toBigInt();
      // debug(
      //   accountId,
      //   printTokens(context.polkadotApi, candidate[1].unwrap().totalCounted.toBigInt(), 3, 9),
      //   printTokens(context.polkadotApi, expectedTotalCounted, 3, 9)
      // );
      expect(candidate[1].unwrap().totalCounted.toBigInt()).to.equal(expectedTotalCounted);
    }

    debug(
      `Verified ${Object.keys(allCandidateInfo).length} candidates and ${
        allDelegatorState.length
      } delegators`
    );
  });

  it("candidate topDelegator total matches the sum", async function () {
    for (const topDelegation of allTopDelegations) {
      expect(topDelegation[1].unwrap().total.toBigInt()).to.equal(
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
    this.timeout(120000);
    // Load data
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

      expect(topDelegations.delegations.length).to.equal(topDelegators.length);

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
        expect(topDelegators[index].delegation.amount.toBigInt()).to.equal(
          topDelegations.delegations[index].amount.toBigInt()
        );
      }
    }

    debug(
      `Verified ${Object.keys(allCandidateInfo).length} candidates and ${
        allDelegatorState.length
      } delegators`
    );
  });
});
