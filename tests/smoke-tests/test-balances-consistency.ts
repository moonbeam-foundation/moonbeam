import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import chalk from "chalk";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:balances");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify balances consistency`, { wssUrl, relayWssUrl }, (context) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;

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
    specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();

    if (process.env.ACCOUNT_ID) {
      const userId = process.env.ACCOUNT_ID.toLowerCase();
      accounts[userId] = await apiAt.query.system.account(userId);
      return;
    }

    // loop over all system accounts
    while (true) {
      let query = await apiAt.query.system.account.entriesPaged({
        args: [],
        pageSize: limit,
        startKey: last_key,
      });

      if (query.length == 0) {
        break;
      }
      count += query.length;

      for (const user of query) {
        let accountId = `0x${user[0].toHex().slice(-40)}`;
        last_key = user[0].toString();
        accounts[accountId] = user[1];
      }
      if (count % (10 * limit) == 0) {
        debug(`Retrieved ${count} accounts`);
      }
    }
    debug(`Retrieved ${count} total accounts`);
  });

  it("should have matching deposit/reserved", async function () {
    this.timeout(240000);
    // Load data
    const [
      proxies,
      proxyAnnouncements,
      treasuryProposals,
      mappingWithDeposit,
      candidateInfo,
      delegatorState,
      identities,
      subIdentities,
      democracyDeposits,
      democracyVotes,
      preimages,
      assets,
      assetsMetadata,
      localAssets,
      localAssetsMetadata,
      localAssetDeposits,
      namedReserves,
      locks,
      delegatorStakingMigrations,
      collatorStakingMigrations,
    ] = await Promise.all([
      apiAt.query.proxy.proxies.entries(),
      apiAt.query.proxy.announcements.entries(),
      apiAt.query.treasury.proposals.entries(),
      apiAt.query.authorMapping.mappingWithDeposit.entries(),
      apiAt.query.parachainStaking.candidateInfo.entries(),
      apiAt.query.parachainStaking.delegatorState.entries(),
      apiAt.query.identity.identityOf.entries(),
      apiAt.query.identity.subsOf.entries(),
      apiAt.query.democracy.depositOf.entries(),
      apiAt.query.democracy.votingOf.entries(),
      apiAt.query.democracy.preimages.entries(),
      apiAt.query.assets.asset.entries(),
      apiAt.query.assets.metadata.entries(),
      apiAt.query.localAssets.asset.entries(),
      apiAt.query.localAssets.metadata.entries(),
      apiAt.query.assetManager.localAssetDeposit.entries(),
      apiAt.query.balances.reserves.entries(),
      apiAt.query.balances.locks.entries(),
      specVersion >= 1700 && specVersion < 1800
        ? apiAt.query.parachainStaking.delegatorReserveToLockMigrations.entries()
        : [],
      specVersion >= 1700 && specVersion < 1800
        ? apiAt.query.parachainStaking.collatorReserveToLockMigrations.entries()
        : [],
    ]);

    const delegatorStakingMigrationAccounts = delegatorStakingMigrations.reduce(
      (p, migration: any) => {
        if (migration[1].isTrue) {
          p[`0x${migration[0].toHex().slice(-40)}`] = true;
        }
        return p;
      },
      {} as any
    ) as { [account: string]: boolean };

    const collatorStakingMigrationAccounts = collatorStakingMigrations.reduce(
      (p, migration: any) => {
        if (migration[1].isTrue) {
          p[`0x${migration[0].toHex().slice(-40)}`] = true;
        }
        return p;
      },
      {} as any
    ) as { [account: string]: boolean };

    const expectedReserveByAccount: {
      [accountId: string]: { total: bigint; reserved: { [key: string]: bigint } };
    } = [
      treasuryProposals.map((proposal) => ({
        accountId: `0x${proposal[1].unwrap().proposer.toHex().slice(-40)}`,
        reserved: {
          treasury: proposal[1].unwrap().bond.toBigInt(),
        },
      })),
      proxies.map((proxy) => ({
        accountId: `0x${proxy[0].toHex().slice(-40)}`,
        reserved: {
          proxy: proxy[1][1].toBigInt(),
        },
      })),
      proxyAnnouncements.map((announcement) => ({
        accountId: `0x${announcement[0].toHex().slice(-40)}`,
        reserved: {
          announcement: announcement[1][1].toBigInt(),
        },
      })),
      mappingWithDeposit.map((mapping) => ({
        accountId: `0x${mapping[1].unwrap().account.toHex().slice(-40)}`,
        reserved: {
          mapping: mapping[1].unwrap().deposit.toBigInt(),
        },
      })),
      candidateInfo
        .map((candidate) =>
          // Support the case of the migration in 1700
          specVersion < 1700 ||
          !collatorStakingMigrationAccounts[`0x${candidate[0].toHex().slice(-40)}`]
            ? {
                accountId: `0x${candidate[0].toHex().slice(-40)}`,
                reserved: {
                  candidate: candidate[1].unwrap().bond.toBigInt(),
                },
              }
            : null
        )
        .filter((r) => !!r),
      ,
      delegatorState
        .map((delegator) =>
          // Support the case of the migration in 1700
          specVersion < 1700 ||
          !delegatorStakingMigrationAccounts[`0x${delegator[0].toHex().slice(-40)}`]
            ? {
                accountId: `0x${delegator[0].toHex().slice(-40)}`,
                reserved: {
                  delegator: delegator[1].unwrap().total.toBigInt(),
                },
              }
            : null
        )
        .filter((r) => !!r),
      identities.map((identity) => ({
        accountId: `0x${identity[0].toHex().slice(-40)}`,
        reserved: {
          identity: identity[1].unwrap().deposit.toBigInt(),
          requestJudgements: identity[1]
            .unwrap()
            .judgements.reduce(
              (acc, value) => acc + ((value[1].isFeePaid && value[1].asFeePaid.toBigInt()) || 0n),
              0n
            ),
        },
      })),
      subIdentities.map((subIdentity) => ({
        accountId: `0x${subIdentity[0].toHex().slice(-40)}`,
        reserved: {
          identity: subIdentity[1][0].toBigInt(),
        },
      })),
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
                    democratyDeposit: 0n,
                  },
                };
              }
              p[deposit.accountId].reserved.democratyDeposit += deposit.reserved;
              return p;
            },
            {} as {
              [accountId: string]: { accountId: string; reserved: { [key: string]: bigint } };
            }
          )
      ),
      preimages
        .filter((preimage) => preimage[1].unwrap().isAvailable)
        .map((preimage) => ({
          accountId: preimage[1].unwrap().asAvailable.provider.toHex(),
          reserved: {
            preimage: preimage[1].unwrap().asAvailable.deposit.toBigInt(),
          },
        })),
      assets.map((asset) => ({
        accountId: `0x${asset[1].unwrap().owner.toHex().slice(-40)}`,
        reserved: {
          asset: asset[1].unwrap().deposit.toBigInt(),
        },
      })),
      assetsMetadata.map((assetMetadata) => ({
        accountId: `0x${assets
          .find((asset) => asset[0].toHex().slice(-64) == assetMetadata[0].toHex().slice(-64))[1]
          .unwrap()
          .owner.toHex()
          .slice(-40)}`,
        reserved: {
          assetMetadata: assetMetadata[1].deposit.toBigInt(),
        },
      })),
      localAssets.map((localAsset) => ({
        accountId: `0x${localAsset[1].unwrap().owner.toHex().slice(-40)}`,
        reserved: {
          localAsset: localAsset[1].unwrap().deposit.toBigInt(),
        },
      })),
      localAssetsMetadata.map((localAssetMetadata) => ({
        accountId: `0x${localAssets
          .find(
            (localAsset) =>
              localAsset[0].toHex().slice(-64) == localAssetMetadata[0].toHex().slice(-64)
          )[1]
          .unwrap()
          .owner.toHex()
          .slice(-40)}`,
        reserved: {
          localAssetMetadata: localAssetMetadata[1].deposit.toBigInt(),
        },
      })),
      localAssetDeposits.map((assetDeposit) => ({
        accountId: assetDeposit[1].unwrap().creator.toHex(),
        reserved: {
          localAssetDeposit: assetDeposit[1].unwrap().deposit.toBigInt(),
        },
      })),
      namedReserves.map((namedReservesOf) => ({
        accountId: `0x${namedReservesOf[0].toHex().slice(-40)}`,
        reserved: {
          named: namedReservesOf[1]
            .map((namedDeposit) => namedDeposit.amount.toBigInt())
            .reduce((accumulator, curr) => accumulator + curr),
        },
      })),
    ]
      .flat()
      .reduce((p, v) => {
        if (!p[v.accountId]) {
          p[v.accountId] = {
            total: 0n,
            reserved: {},
          };
        }
        p[v.accountId].total += Object.keys(v.reserved).reduce((p, key) => p + v.reserved[key], 0n);
        p[v.accountId].reserved = { ...p[v.accountId].reserved, ...v.reserved };
        return p;
      }, {} as { [key: string]: { total: bigint; reserved: { [key: string]: bigint } } });

    debug(`Retrieved ${Object.keys(expectedReserveByAccount).length} deposits`);

    const failedReserved = [];

    for (const accountId of Object.keys(accounts)) {
      let reserved = accounts[accountId].data.reserved.toBigInt();
      const expectedReserve = expectedReserveByAccount[accountId]?.total || 0n;

      if (reserved != expectedReserve) {
        failedReserved.push(
          `${accountId} (reserved: ${reserved} vs expected: ${expectedReserve})\n` +
            `        (${Object.keys(expectedReserveByAccount[accountId]?.reserved || {})
              .map(
                (key) =>
                  `${key}: ${printTokens(
                    context.polkadotApi,
                    expectedReserveByAccount[accountId].reserved[key]
                  )}`
              )
              .join(` - `)})`
        );
      }
    }

    const expectedLocksByAccount: {
      [accountId: string]: { [id: string]: bigint };
    } = [
      candidateInfo
        .map((candidate) =>
          // Support the case of the migration in 1700
          specVersion >= 1800 ||
          collatorStakingMigrationAccounts[`0x${candidate[0].toHex().slice(-40)}`]
            ? {
                accountId: `0x${candidate[0].toHex().slice(-40)}`,
                locks: {
                  ColStake: candidate[1].unwrap().bond.toBigInt(),
                },
              }
            : null
        )
        .filter((r) => !!r),
      ,
      delegatorState
        .map((delegator) =>
          // Support the case of the migration in 1700
          specVersion >= 1800 ||
          delegatorStakingMigrationAccounts[`0x${delegator[0].toHex().slice(-40)}`]
            ? {
                accountId: `0x${delegator[0].toHex().slice(-40)}`,
                locks: {
                  DelStake: delegator[1].unwrap().total.toBigInt(),
                },
              }
            : null
        )
        .filter((r) => !!r),
      ,
      democracyVotes
        .map(
          (votes) =>
            votes[1].isDirect
              ? {
                  accountId: `0x${votes[0].toHex().slice(-40)}`,
                  locks: {
                    democrac: votes[1].asDirect.votes.reduce((p, v) => {
                      const value = v[1].isStandard
                        ? v[1].asStandard.balance.toBigInt()
                        : v[1].asSplit.aye.toBigInt() + v[1].asSplit.nay.toBigInt();
                      return p > value ? p : value;
                    }, 0n),
                  },
                }
              : null // Not sure if in isDelegation should the balance be counted to the delegator ?
        )
        .filter((d) => !!d),
    ]
      .flat()
      .reduce(
        (p, v) => {
          if (!p[v.accountId]) {
            p[v.accountId] = {};
          }
          p[v.accountId] = { ...p[v.accountId], ...v.locks };
          return p;
        },
        {} as {
          [accountId: string]: { [id: string]: bigint };
        }
      );
    debug(`Retrieved ${Object.keys(expectedLocksByAccount).length} accounts with locks`);

    const failedLocks = [];
    const locksByAccount = locks.reduce((p, lockSet) => {
      p[`0x${lockSet[0].toHex().slice(-40)}`] = Object.values(lockSet[1].toArray()).reduce(
        (p, lock) => ({
          ...p,
          [lock.id.toHuman().toString()]: lock.amount.toBigInt(),
        }),
        {}
      );
      return p;
    }, {} as { [account: string]: { [id: string]: bigint } });

    for (const accountId of new Set([
      ...Object.keys(locksByAccount),
      ...Object.keys(expectedLocksByAccount),
    ])) {
      const locks = locksByAccount[accountId] || {};
      const expectedLocks = expectedLocksByAccount[accountId] || {};

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

    if (failedLocks.length > 0 || failedReserved.length > 0) {
      if (failedReserved.length > 0) {
        console.log("Failed accounts reserves");
        console.log(chalk.red(failedReserved.join("\n")));
      }
      if (failedLocks.length > 0) {
        console.log("Failed accounts locks");
        console.log(chalk.red(failedLocks.join("\n")));
      }
      expect(failedReserved.length, "Failed accounts reserves").to.equal(0);
      expect(failedLocks.length, "Failed accounts locks").to.equal(0);
    }

    debug(`Verified ${Object.keys(accounts).length} total reserved balance (at #${atBlockNumber})`);
  });

  it("should match total supply", async function () {
    const totalIssuance = await apiAt.query.balances.totalIssuance();

    expect(
      Object.keys(accounts).reduce(
        (p, accountId) =>
          accounts[accountId].data.free.toBigInt() +
          accounts[accountId].data.reserved.toBigInt() +
          p,
        0n
      )
    ).to.equal(totalIssuance.toBigInt());
    debug(`Verified total issuance`);
  });
});
