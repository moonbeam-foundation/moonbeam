import "@moonbeam-network/api-augment/moonbase";
import { ApiDecoration } from "@polkadot/api/types";
import { H256 } from "@polkadot/types/interfaces/runtime";
import { u32 } from "@polkadot/types";
import { AccountId20 } from "@polkadot/types/interfaces/runtime";
import type {
  FrameSystemAccountInfo,
  PalletReferendaDeposit,
  PalletBalancesBalanceLock,
  PalletPreimageRequestStatus,
} from "@polkadot/types/lookup";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { Option } from "@polkadot/types-codec";
import { StorageKey } from "@polkadot/types";
import { extractPreimageDeposit } from "../util/block";
import { rateLimiter } from "../util/common";
const debug = require("debug")("smoke:balances");

enum ReserveType {
  Treasury = "1",
  Proxy = "2",
  Announcement = "3",
  Mapping = "4",
  Candidate = "5",
  Delegator = "6",
  RequestJudgements = "7",
  Identity = "8",
  DemocracyDeposit = "9",
  Preimage = "10",
  ReferendumInfo = "11",
  Asset = "12",
  AssetMetadata = "13",
  LocalAsset = "14",
  LocalAssetMetadata = "15",
  LocalAssetDeposit = "16",
  Named = "17",
}

const  getReserveTypeByValue = ( value: string): string | null => {
  for (const key in ReserveType) {
    if (ReserveType[key] === value) {
      return key;
    }
  }
  return null;
}

type ReservedInfo = { total?: bigint; reserved?: { [key: string]: bigint } };

describeSmokeSuite("S300", `Verifying balances consistency`, (context, testIt) => {
  // const accounts: { [account: string]: FrameSystemAccountInfo } = {};
  const accountMap = new Map<string, FrameSystemAccountInfo>();
  const limiter = rateLimiter();

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;
  let runtimeName: string;
  let candidateInfo;
  let collatorStakingMigrations;
  let delegatorState;
  let delegatorStakingMigrations;
  let delegatorStakingMigrationAccounts;
  let collatorStakingMigrationAccounts;

  const accountIdTo20 = (accountId: string) =>
    context.polkadotApi.createType("AccountId20", accountId.toLowerCase()).toString();

  before("Retrieve all balances", async function () {
    // It takes time to load all the accounts.
    this.timeout(3600000); // 1 hour should be enough
    const limit = 1000;
    let last_key = "";
    let count = 0;

    atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    specVersion = apiAt.consts.system.version.specVersion.toNumber();
    runtimeName = apiAt.runtimeVersion.specName.toString();

    if (process.env.ACCOUNT_ID) {
      const userId = process.env.ACCOUNT_ID;
      accountMap.set(accountIdTo20(userId), await apiAt.query.system.account(userId));
      return;
    }

    // loop over all system accounts
    while (true) {
      const query = await limiter.schedule(() =>
        apiAt.query.system.account.entriesPaged({
          args: [],
          pageSize: limit,
          startKey: last_key,
        })
      );

      if (query.length === 0) {
        break;
      }

      count += query.length;

      for (const user of query) {
        last_key = user[0].toString();
        const accountId = `0x${user[0].toHex().slice(-40)}`;
        accountMap.set(accountIdTo20(accountId), user[1]);
      }
      if (count % (10 * limit) == 0) {
        debug(`Retrieved ${count} accounts`);
      }
    }
    debug(`Retrieved ${count} total accounts`);

    candidateInfo = await apiAt.query.parachainStaking.candidateInfo.entries();

    collatorStakingMigrations =
      specVersion >= 1700 && specVersion < 1800
        ? await apiAt.query.parachainStaking.collatorReserveToLockMigrations.entries()
        : [];

    delegatorState = await apiAt.query.parachainStaking.delegatorState.entries();

    (delegatorStakingMigrations =
      specVersion >= 1700 && specVersion < 1800
        ? await apiAt.query.parachainStaking.delegatorReserveToLockMigrations.entries()
        : []),
      (delegatorStakingMigrationAccounts = delegatorStakingMigrations.reduce(
        (p, migration: any) => {
          if (migration[1].isTrue) {
            p[`0x${migration[0].toHex().slice(-40)}`] = true;
          }
          return p;
        },
        {} as any
      ) as { [account: string]: boolean });

    collatorStakingMigrationAccounts = collatorStakingMigrations.reduce((p, migration: any) => {
      if (migration[1].isTrue) {
        p[`0x${migration[0].toHex().slice(-40)}`] = true;
      }
      return p;
    }, {} as any) as { [account: string]: boolean };
  });

  testIt("C100", `should have matching deposit/reserved`, async function () {
    this.timeout(240000);
    const [
      proxies,
      proxyAnnouncements,
      treasuryProposals,
      mappingWithDeposit,
      identities,
      subIdentities,
      democracyDeposits,
      democracyPreimages, // TODO add this check back to map
      preimageStatuses,
      referendumInfoFor,
      assets,
      assetsMetadata,
      localAssets,
      localAssetsMetadata,
      localAssetDeposits,
      namedReserves,
    ] = await Promise.all([
      apiAt.query.proxy.proxies.entries(),
      apiAt.query.proxy.announcements.entries(),
      apiAt.query.treasury.proposals.entries(),
      apiAt.query.authorMapping.mappingWithDeposit.entries(),
      apiAt.query.identity.identityOf.entries(),
      apiAt.query.identity.subsOf.entries(),
      apiAt.query.democracy.depositOf.entries(),
      specVersion < 2000
        ? apiAt.query.democracy.preimages.entries()
        : ([] as [StorageKey<[H256]>, Option<any>][]),
      (specVersion >= 1900 && runtimeName == "moonbase") || specVersion >= 2000
        ? apiAt.query.preimage.statusFor.entries()
        : ([] as [StorageKey<[H256]>, Option<PalletPreimageRequestStatus>][]),
      (specVersion >= 1900 && runtimeName == "moonbase") ||
      (specVersion >= 2100 && runtimeName == "moonriver")
        ? apiAt.query.referenda.referendumInfoFor.entries()
        : ([] as [StorageKey<[u32]>, Option<any>][]),
      apiAt.query.assets.asset.entries(),
      apiAt.query.assets.metadata.entries(),
      apiAt.query.localAssets.asset.entries(),
      apiAt.query.localAssets.metadata.entries(),
      apiAt.query.assetManager.localAssetDeposit.entries(),
      apiAt.query.balances.reserves.entries(),
    ]);

    // TODO: make each check blob so we throw away the query bit after each save into the DB

    const expectedReserveMap = new Map<string, ReservedInfo>();

    const updateReserveMap = (
      account: string,
      newReserve: {
        [key: string]: bigint;
      }
    ) => {
      let isZero = true;
      for (const key in newReserve) {
        if (newReserve[key] > 0n) {
          isZero = false;
        }
      }
      if (isZero) {
        return;
      }
      const account20 = accountIdTo20(account);

      const value = expectedReserveMap.get(account20);
      if (value === undefined) {
        expectedReserveMap.set(account20, {
          total: 0n,
          reserved: newReserve,
        });
        return;
      }
 

      const newReserved = { ...value.reserved, ...newReserve };
      const newTotal = value.total;

      expectedReserveMap.set(account20, {
        total: newTotal,
        reserved: newReserved,
      });
    };

    treasuryProposals.forEach((proposal) =>
      updateReserveMap(`0x${proposal[1].unwrap().proposer.toHex().slice(-40)}`, {
        [ReserveType.Treasury]: proposal[1].unwrap().bond.toBigInt(),
      })
    );

    proxies.forEach((proxy) => {
      // updateReserveMap(`0x${proxy[0].toHex().slice(-40)}`, {
      //   [ReserveType.Proxy]: proxy[1][1].toBigInt(),
      // });
    });

    proxyAnnouncements.forEach((announcement) =>
      updateReserveMap(`0x${announcement[0].toHex().slice(-40)}`, {
        [ReserveType.Announcement]: announcement[1][1].toBigInt(),
      })
    );

    mappingWithDeposit.forEach((mapping) =>
      updateReserveMap(`0x${mapping[1].unwrap().account.toHex().slice(-40)}`, {
        [ReserveType.Mapping]: mapping[1].unwrap().deposit.toBigInt(),
      })
    );

    candidateInfo.forEach((candidate) => {
      if (
        specVersion < 1700 ||
        (specVersion < 1800 &&
          !collatorStakingMigrationAccounts[`0x${candidate[0].toHex().slice(-40)}`])
      ) {
        updateReserveMap(`0x${candidate[0].toHex().slice(-40)}`, {
          [ReserveType.Candidate]: candidate[1].unwrap().bond.toBigInt(),
        });
      }
    });

    delegatorState.forEach((delegator) => {
      if (
        specVersion < 1700 ||
        (specVersion < 1800 &&
          !delegatorStakingMigrationAccounts[`0x${delegator[0].toHex().slice(-40)}`])
      ) {
        updateReserveMap(`0x${delegator[0].toHex().slice(-40)}`, {
          [ReserveType.Delegator]: delegator[1].unwrap().total.toBigInt(),
        });
      }
    });

    identities.forEach((identity) => {
      updateReserveMap(`0x${identity[0].toHex().slice(-40)}`, {
        [ReserveType.Identity]: identity[1].unwrap().deposit.toBigInt(),
        [ReserveType.RequestJudgements]: identity[1]
          .unwrap()
          .judgements.reduce(
            (acc, value) => acc + ((value[1].isFeePaid && value[1].asFeePaid.toBigInt()) || 0n),
            0n
          ),
      });
    });

    subIdentities.forEach((subIdentity) => {
      updateReserveMap(`0x${subIdentity[0].toHex().slice(-40)}`, {
        [ReserveType.Identity]: subIdentity[1][0].toBigInt(),
      });
    });

    democracyDeposits
      .map((depositOf) =>
        depositOf[1]
          .unwrap()[0]
          .map((deposit) => ({
            accountId: deposit.toHex(),
            reserved: depositOf[1].unwrap()[1].toBigInt(),
          }))
          .flat()
          .reduce((p, deposit) => {
            // We merge multiple reserves together for same account
            if (!p[deposit.accountId]) {
              p[deposit.accountId] = {
                accountId: deposit.accountId,
                reserved: {
                  [ReserveType.DemocracyDeposit]: 0n,
                },
              };
            }
            p[deposit.accountId].reserved[ReserveType.DemocracyDeposit] += deposit.reserved;
            return p;
          }, {})
      )
      .forEach((deposit: any) => {
        updateReserveMap(deposit.accountId, deposit.reserved);
      });

    Object.values(
      democracyDeposits
        .map((depositOf) =>
          depositOf[1].unwrap()[0].map((deposit) => ({
            accountId: deposit.toHex(),
            reserved: depositOf[1].unwrap()[1].toBigInt(),
          }))
        )
        .flat()
        .reduce(
          (p, deposit) => {
            // We merge multiple reserves together for same account
            if (!p[deposit.accountId]) {
              p[deposit.accountId] = {
                accountId: deposit.accountId,
                reserved: {
                  [ReserveType.DemocracyDeposit]: 0n,
                },
              };
            }
            p[deposit.accountId].reserved[ReserveType.DemocracyDeposit] += deposit.reserved;
            return p;
          },
          {} as {
            [accountId: string]: { accountId: string; reserved: { [key: string]: bigint } };
          }
        )
    ).forEach((deposit: any) => {
      updateReserveMap(deposit.accountId, deposit.reserved);
    });

    democracyPreimages
      .filter((preimg: any) => preimg[1].unwrap().isAvailable)
      .forEach((preimage: any) => {
        updateReserveMap(preimage[1].unwrap().asAvailable.provider.toHex(), {
          [ReserveType.Preimage]: preimage[1].unwrap().asAvailable.deposit.toBigInt(),
        });
      });

    preimageStatuses
      .filter((status) => status[1].unwrap().isUnrequested || status[1].unwrap().isRequested)
      .forEach((status) => {
        const deposit = extractPreimageDeposit(
          status[1].unwrap().isUnrequested
            ? status[1].unwrap().asUnrequested
            : status[1].unwrap().asRequested
        );
        updateReserveMap(deposit.accountId, {
          [ReserveType.Preimage]: deposit.amount !== 0n ? deposit.amount.toBigInt() : 0n,
        });
      });

    referendumInfoFor.forEach((info) => {
      const deposits = (
        info[1].unwrap().isApproved
          ? [info[1].unwrap().asApproved[1], info[1].unwrap().asApproved[2].unwrapOr(null)]
          : info[1].unwrap().isRejected
          ? [info[1].unwrap().asRejected[1], info[1].unwrap().asRejected[2].unwrapOr(null)]
          : info[1].unwrap().isCancelled
          ? [info[1].unwrap().asCancelled[1], info[1].unwrap().asCancelled[2].unwrapOr(null)]
          : info[1].unwrap().isTimedOut
          ? [info[1].unwrap().asTimedOut[1], info[1].unwrap().asTimedOut[2].unwrapOr(null)]
          : info[1].unwrap().isOngoing
          ? [
              info[1].unwrap().asOngoing.submissionDeposit,
              info[1].unwrap().asOngoing.decisionDeposit.unwrapOr(null),
            ]
          : ([] as PalletReferendaDeposit[])
      ).filter((value) => !!value && !value.isNone);

      deposits.forEach((deposit) => {
        // Support for https://github.com/paritytech/substrate/pull/12788
        // which make deposit optional.
        // TODO: better handle unwrapping

        updateReserveMap((deposit.unwrap ? deposit.unwrap() : deposit).who.toHex(), {
          [ReserveType.ReferendumInfo]: (deposit.unwrap ? deposit.unwrap() : deposit).amount.toBigInt(),
        });
      });
    });

    assets.forEach((asset) => {
      updateReserveMap(`0x${asset[1].unwrap().owner.toHex().slice(-40)}`, {
        [ReserveType.Asset]: asset[1].unwrap().deposit.toBigInt(),
      });
    });

    assetsMetadata.forEach((assetMetadata) => {
      updateReserveMap(
        `0x${assets
          .find((asset) => asset[0].toHex().slice(-64) == assetMetadata[0].toHex().slice(-64))[1]
          .unwrap()
          .owner.toHex()
          .slice(-40)}`,
        {
          [ReserveType.AssetMetadata]: assetMetadata[1].deposit.toBigInt(),
        }
      );
    });

    localAssets.forEach((localAsset) => {
      updateReserveMap(`0x${localAsset[1].unwrap().owner.toHex().slice(-40)}`, {
        [ReserveType.LocalAsset]: localAsset[1].unwrap().deposit.toBigInt(),
      });
    });

    localAssetsMetadata.forEach((localAssetMetadata) => {
      updateReserveMap(
        `0x${localAssets
          .find(
            (localAsset) =>
              localAsset[0].toHex().slice(-64) == localAssetMetadata[0].toHex().slice(-64)
          )[1]
          .unwrap()
          .owner.toHex()
          .slice(-40)}`,
        {
          [ReserveType.LocalAssetMetadata]: localAssetMetadata[1].deposit.toBigInt(),
        }
      );
    });

    localAssetDeposits.forEach((assetDeposit) => {
      updateReserveMap(assetDeposit[1].unwrap().creator.toHex(), {
        [ReserveType.LocalAssetDeposit]: assetDeposit[1].unwrap().deposit.toBigInt(),
      });
    });

    namedReserves.forEach((namedReservesOf) => {
      updateReserveMap(`0x${namedReservesOf[0].toHex().slice(-40)}`, {
        [ReserveType.Named]: namedReservesOf[1]
          .map((namedDeposit) => namedDeposit.amount.toBigInt())
          .reduce((accumulator, curr) => accumulator + curr),
      });
    });

    debug(`Retrieved ${expectedReserveMap.size} deposits`);

    expectedReserveMap.forEach(({ reserved }, key) => {
      const total = Object.values(reserved).reduce((total, amount) => {
        total += amount;
        return total;
      }, 0n);
      expectedReserveMap.set(key, { reserved, total });
    });

    const failedReserved = [];

    for (const accountId of accountMap.keys()) {
      const reserved = accountMap.has(accountId)
        ? accountMap.get(accountId).data.reserved.toBigInt()
        : 0n;
      const expectedReserve = expectedReserveMap.has(accountId)
        ? expectedReserveMap.get(accountId).total
        : 0n;

      if (reserved != expectedReserve) {
        failedReserved.push(
          `⚠️  ${accountId.toString()} (reserved: ${reserved} vs expected: ${expectedReserve})\n` +
            `\tℹ️  Expected only contains: (${Object.keys(
              (expectedReserveMap.has(accountId) && expectedReserveMap.get(accountId).reserved) ||
                {}
            )
              .map(
                (key) =>
                  `${getReserveTypeByValue(key)}: ${printTokens(
                    context.polkadotApi,
                    expectedReserveMap.get(accountId).reserved[key]
                  )}`
              )
              .join(` - `)})`
        );
      }
    }

    if (failedReserved.length > 0) {
      debug("Failed accounts reserves");
    }

    expect(
      failedReserved.length,
      `❌ Mismatched account reserves: \n${failedReserved.join(",\n")}`
    ).to.equal(0);

    debug(`Verified ${accountMap.size} total reserved balance (at #${atBlockNumber})`);
  });

  testIt("C200", "should match total locks", async function () {
    this.timeout(30000);
    const locks = await apiAt.query.balances.locks.entries();
    const democracyVotes = await apiAt.query.democracy.votingOf.entries();
    const expectedLocksMap = new Map<
      string,
      { total?: bigint; locks?: { [key: string]: bigint } }
    >();

    const updateExpectedLocksMap = (account: string, lock: { [key: string]: bigint }) => {
      const value = expectedLocksMap.get(account);
      if (value === undefined) {
        expectedLocksMap.set(account, { total: 0n, locks: lock });
        return;
      }
      const updatedLocks = { ...value.locks, ...lock };
      const newTotal = value.total;
      expectedLocksMap.set(account, { total: newTotal, locks: updatedLocks });
    };

    candidateInfo.forEach((candidate) => {
      // Support the case of the migration in 1700
      if (
        specVersion >= 1800 ||
        collatorStakingMigrationAccounts[`0x${candidate[0].toHex().slice(-40)}`]
      ) {
        updateExpectedLocksMap(`0x${candidate[0].toHex().slice(-40)}`, {
          ColStake: candidate[1].unwrap().bond.toBigInt(),
        });
      }
    });

    delegatorState.forEach((delegator) => {
      // Support the case of the migration in 1700
      if (
        specVersion >= 1800 ||
        delegatorStakingMigrationAccounts[`0x${delegator[0].toHex().slice(-40)}`]
      ) {
        updateExpectedLocksMap(`0x${delegator[0].toHex().slice(-40)}`, {
          DelStake: delegator[1].unwrap().total.toBigInt(),
        });
      }
    });

    democracyVotes.forEach((votes) => {
      if (votes[1].isDirect) {
        updateExpectedLocksMap(`0x${votes[0].toHex().slice(-40)}`, {
          democracy: votes[1].asDirect.votes.reduce((p, v) => {
            const value = v[1].isStandard
              ? v[1].asStandard.balance.toBigInt()
              : v[1].asSplit.aye.toBigInt() + v[1].asSplit.nay.toBigInt();
            return p > value ? p : value;
          }, 0n),
        });
      }

      // Not sure if in isDelegation should the balance be counted to the delegator ?
    });

    debug(`Retrieved ${expectedLocksMap.size} accounts with locks`);

    expectedLocksMap.forEach(({ locks }, key) => {
      const total = Object.values(locks).reduce((total, amount) => {
        total += amount;
        return total;
      }, 0n);
      expectedLocksMap.set(key, { locks, total });
    });

    const failedLocks = [];
    const locksByAccount = locks.reduce((p, lockSet) => {
      p[`0x${lockSet[0].toHex().slice(-40)}`] = Object.values(lockSet[1].toArray()).reduce(
        (p, lock) => ({
          ...(p as any),
          [(lock as any).id.toHuman().toString()]: (lock as any).amount.toBigInt(),
        }),
        {}
      );
      return p;
    }, {} as { [account: string]: { [id: string]: bigint } });

    for (const accountId of Object.keys(locksByAccount)) {
      const locks = locksByAccount[accountId] || {};
      // const expectedLocks = expectedLocksByAccount[accountId] || {};
      const expectedLocks = expectedLocksMap.get(accountId).locks;

      for (const key of new Set([...Object.keys(expectedLocks), ...Object.keys(locks)])) {
        if (expectedLocks[key] > locks[key]) {
          failedLocks.push(
            `${accountId} (lock ${key}: actual ${
              locks[key] && printTokens(context.polkadotApi, locks[key])
            } < expected: ${
              (expectedLocks[key] && printTokens(context.polkadotApi, expectedLocks[key])) || ""
            })\n ${[...new Set([...Object.keys(expectedLocks), ...Object.keys(locks)])]
              .map(
                (key) =>
                  `         - ${key}: actual ${(locks[key] || "")
                    .toString()
                    .padStart(23, " ")} - ${(expectedLocks[key] || "")
                    .toString()
                    .padStart(23, " ")}`
              )
              .join("\n")}`
          );
        }
      }
    }

    if (failedLocks.length > 0) {
      debug("Failed accounts locks");
    }
    expect(failedLocks.length, `Failed accounts locks: ${failedLocks.join(", ")}`).to.equal(0);
  });

  testIt("C300", `should match total supply`, async function () {
    this.timeout(30000);
    const totalIssuance = await apiAt.query.balances.totalIssuance();

    expect(
      Array.from(accountMap.keys()).reduce(
        (p, accountId) =>
          accountMap.get(accountId).data.free.toBigInt() +
          accountMap.get(accountId).data.reserved.toBigInt() +
          p,
        0n
      )
    ).to.equal(totalIssuance.toBigInt());
    debug(`Verified total issuance`);
  });
});
