import "@moonbeam-network/api-augment/moonbase";
import { ApiDecoration } from "@polkadot/api/types";
import {  u16 } from "@polkadot/types";
import { AccountId20 } from "@polkadot/types/interfaces";
import type {
  PalletReferendaDeposit,
  PalletConvictionVotingVoteVoting,
} from "@polkadot/types/lookup";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { StorageKey } from "@polkadot/types";
import { extractPreimageDeposit } from "../util/block";
import { rateLimiter } from "../util/common";
import { ONE_HOURS } from "../util/constants";

const debug = require("debug")("smoke:balances");

enum ReserveType {
  Treasury = "1",
  Proxy = "2",
  Announcement = "3",
  AuthorMapping = "4",
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
  SubIdentity = "18",
  PreimageStatus = "19",
}

type ReservedInfo = { total?: bigint; reserved?: { [key: string]: bigint } };
type LocksInfo = { total?: bigint; locks?: { [key: string]: bigint } };

// This test attemps to reconcile the total amount of locked tokens across the entire
// chain by ensuring that individual storages match the reserved balances and locks.
// In order to not exhaust memory, the expected results are calculated first and then
// All system accounts are iterated over, without storing them, to ensure memory
// is not exhausted.

// This test is limited by Polkadot{Js}.api 's DecodeStruct functionality,
// and works best for networks < 2million accounts.
// https://github.com/polkadot-js/api/issues/5615

describeSmokeSuite("S300", `Verifying balances consistency`, (context, testIt) => {
  const expectedReserveMap = new Map<string, ReservedInfo>();
  const expectedLocksMap = new Map<string, LocksInfo>();
  const locksMap = new Map<string, { total: bigint }>();
  const limiter = rateLimiter();
  let failedLocks = [];
  let failedReserved = [];
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;
  let runtimeName: string;
  let totalAccounts: bigint = 0n;
  let totalIssuance: bigint = 0n;
  let symbol: string;

  // Test Case Specific Helper Functions
  const hexToBase64 = (hex: string): string => {
    const formatted = hex.includes("0x") ? hex.slice(2) : hex;
    return Buffer.from(formatted, "hex").toString("base64");
  };
  const base64ToHex = (base64: string): string => {
    return "0x" + Buffer.from(base64, "base64").toString("hex");
  };

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
    const account64 = hexToBase64(account);
    const value = expectedReserveMap.get(account64);

    if (value === undefined) {
      expectedReserveMap.set(account64, {
        total: 0n,
        reserved: newReserve,
      });
      return;
    }

    let tempRegister;
    Object.keys(newReserve).forEach((key) => {
      if (value.reserved[key]) {
        tempRegister = { [key]: value.reserved[key] + newReserve[key] };
      } else {
        tempRegister = { [key]: newReserve[key] };
      }
    });

    const newReserved = { ...value.reserved, ...tempRegister };
    const newTotal = value.total;

    expectedReserveMap.set(account64, {
      total: newTotal,
      reserved: newReserved,
    });
  };

  const updateExpectedLocksMap = (account: string, lock: { [key: string]: bigint }) => {
    const account64 = hexToBase64(account);
    const value = expectedLocksMap.get(account64);
    if (value === undefined) {
      expectedLocksMap.set(account64, { total: 0n, locks: lock });
      return;
    }
    const updatedLocks = { ...value.locks, ...lock };
    const newTotal = value.total;
    expectedLocksMap.set(account64, { total: newTotal, locks: updatedLocks });
  };

  const getReserveTypeByValue = (value: string): string | null => {
    for (const key in ReserveType) {
      if (ReserveType[key] === value) {
        return key;
      }
    }
    return null;
  };

  before("Retrieve all balances", async function () {
    this.timeout(ONE_HOURS); // 1 hour timeout
    atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    specVersion = apiAt.consts.system.version.specVersion.toNumber();
    runtimeName = apiAt.runtimeVersion.specName.toString();
    symbol = (await context.polkadotApi.rpc.system.properties()).tokenSymbol.unwrap()[0].toString();

    // 1a) Build Expected Results - Reserved Map
    // We iteratively build the expected results by iterating over the storage keys, adding the
    // the amount to a results map

    await new Promise((resolve, reject) => {
      apiAt.query.proxy.proxies
        .entries()
        .then((proxies) => {
          proxies.forEach((proxy) => {
            updateReserveMap(proxy[0].toHex().slice(-40), {
              [ReserveType.Proxy]: proxy[1][1].toBigInt(),
            });
          });
          resolve("proxies scraped");
        })
        .catch((error) => {
          console.error("Error fetching proxies:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    let [democracyVotes, delegatorState, delegatorStakingMigrations, collatorStakingMigrations] =
      await Promise.all([
        apiAt.query.democracy.votingOf.entries(),
        apiAt.query.parachainStaking.delegatorState.entries(),
        specVersion >= 1700 && specVersion < 1800
          ? apiAt.query.parachainStaking.delegatorReserveToLockMigrations.entries()
          : [],
        specVersion >= 1700 && specVersion < 1800
          ? apiAt.query.parachainStaking.collatorReserveToLockMigrations.entries()
          : [],
      ]);

    await new Promise((resolve, reject) => {
      apiAt.query.treasury.proposals
        .entries()
        .then((treasuryProposals) => {
          treasuryProposals.forEach((proposal) => {
            updateReserveMap(proposal[1].unwrap().proposer.toHex().slice(-40), {
              [ReserveType.Treasury]: proposal[1].unwrap().bond.toBigInt(),
            });
          });
          resolve("treasury props scraped");
        })
        .catch((error) => {
          console.error("Error fetching treasury props:", error);
          reject(error); // Reject the outer promise with the error
        });
    });
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

    await new Promise((resolve, reject) => {
      apiAt.query.proxy.announcements
        .entries()
        .then((proxyAnnouncements) => {
          proxyAnnouncements.forEach((announcement) => {
            updateReserveMap(announcement[0].toHex().slice(-40), {
              [ReserveType.Announcement]: announcement[1][1].toBigInt(),
            });
          });
          resolve("proxy announcement scraped");
        })
        .catch((error) => {
          console.error("Error fetching proxy announcement:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.authorMapping.mappingWithDeposit
        .entries()
        .then((mappingWithDeposit) => {
          mappingWithDeposit.forEach((mapping) => {
            updateReserveMap(mapping[1].unwrap().account.toHex().slice(-40), {
              [ReserveType.AuthorMapping]: mapping[1].unwrap().deposit.toBigInt(),
            });
          });
          resolve("author mapping scraped");
        })
        .catch((error) => {
          console.error("Error fetching author mapping:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    // StorageQuery: ParachainStaking.CandidateInfo
    await new Promise((resolve, reject) => {
      apiAt.query.parachainStaking.candidateInfo
        .entries()
        .then(async (candidateInfo) => {
          candidateInfo.forEach((candidate) => {
            if (
              specVersion < 1700 ||
              (specVersion < 1800 &&
                !collatorStakingMigrationAccounts[candidate[0].toHex().slice(-40)])
            ) {
              updateReserveMap(candidate[0].toHex().slice(-40), {
                [ReserveType.Candidate]: candidate[1].unwrap().bond.toBigInt(),
              });
            }
            if (
              specVersion >= 1800 ||
              collatorStakingMigrationAccounts[candidate[0].toHex().slice(-40)]
            ) {
              updateExpectedLocksMap(candidate[0].toHex().slice(-40), {
                ColStake: candidate[1].unwrap().bond.toBigInt(),
              });
            }
          });

          candidateInfo.forEach((candidate) => {
            // Support the case of the migration in 1700
            if (
              specVersion >= 1800 ||
              collatorStakingMigrationAccounts[candidate[0].toHex().slice(-40)]
            ) {
              updateExpectedLocksMap(candidate[0].toHex().slice(-40), {
                ColStake: candidate[1].unwrap().bond.toBigInt(),
              });
            }
          });

          resolve("candidate info scraped");
        })
        .catch((error) => {
          console.error("Error fetching candidate info:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.identity.identityOf
        .entries()
        .then((identities) => {
          identities.forEach((identity) => {
            updateReserveMap(identity[0].toHex().slice(-40), {
              [ReserveType.Identity]: identity[1].unwrap().deposit.toBigInt(),
            });
            updateReserveMap(identity[0].toHex().slice(-40), {
              [ReserveType.RequestJudgements]: identity[1]
                .unwrap()
                .judgements.reduce(
                  (acc, value) =>
                    acc + ((value[1].isFeePaid && value[1].asFeePaid.toBigInt()) || 0n),
                  0n
                ),
            });
          });
          resolve("identities scraped");
        })
        .catch((error) => {
          console.error("Error fetching identities:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.identity.subsOf
        .entries()
        .then((subIdentities) => {
          subIdentities.forEach((subIdentity) => {
            updateReserveMap(subIdentity[0].toHex().slice(-40), {
              [ReserveType.SubIdentity]: subIdentity[1][0].toBigInt(),
            });
          });
          resolve("subIdentities scraped");
        })
        .catch((error) => {
          console.error("Error fetching subIdentities:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.democracy.depositOf
        .entries()
        .then((democracyDeposits) => {
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

          resolve("democracy deposits scraped");
        })
        .catch((error) => {
          console.error("Error fetching democracy deposits:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      if (specVersion < 2000) {
        apiAt.query.democracy.preimages
          .entries()
          .then((democracyPreimages) => {
            democracyPreimages
              .filter((preimg: any) => preimg[1].unwrap().isAvailable)
              .forEach((preimage: any) => {
                updateReserveMap(preimage[1].unwrap().asAvailable.provider.toHex(), {
                  [ReserveType.Preimage]: preimage[1].unwrap().asAvailable.deposit.toBigInt(),
                });
              });
          })
          .catch((error) => {
            console.error("Error fetching democracyPreimages:", error);
            reject(error); // Reject the outer promise with the error
          });
      }
      resolve("democracyPreimages scraped");
    });

    await new Promise((resolve, reject) => {
      if ((specVersion >= 1900 && runtimeName == "moonbase") || specVersion >= 2000) {
        apiAt.query.preimage.statusFor
          .entries()
          .then((preimageStatuses) => {
            preimageStatuses
              .filter(
                (status) => status[1].unwrap().isUnrequested || status[1].unwrap().isRequested
              )
              .map((status) => {
                const deposit = extractPreimageDeposit(
                  status[1].unwrap().isUnrequested
                    ? status[1].unwrap().asUnrequested
                    : status[1].unwrap().asRequested
                );
                return { accountId: deposit.accountId, deposit: deposit.amount };
              })
              .forEach(({ deposit, accountId }) => {
                updateReserveMap(accountId, {
                  [ReserveType.PreimageStatus]: deposit == 0n ? 0n : deposit.toBigInt(),
                });
              });
            resolve("proxies scraped");
          })
          .catch((error) => {
            console.error("Error fetching proxies:", error);
            reject(error); // Reject the outer promise with the error
          });
      }
    });

    await new Promise((resolve, reject) => {
      if (
        (specVersion >= 1900 && runtimeName == "moonbase") ||
        (specVersion >= 2100 && runtimeName == "moonriver")
      ) {
        apiAt.query.referenda.referendumInfoFor
          .entries()
          .then((referendumInfoFor) => {
            referendumInfoFor.forEach((info) => {
              const deposits = (
                info[1].unwrap().isApproved
                  ? [info[1].unwrap().asApproved[1], info[1].unwrap().asApproved[2].unwrapOr(null)]
                  : info[1].unwrap().isRejected
                  ? [info[1].unwrap().asRejected[1], info[1].unwrap().asRejected[2].unwrapOr(null)]
                  : info[1].unwrap().isCancelled
                  ? [
                      info[1].unwrap().asCancelled[1],
                      info[1].unwrap().asCancelled[2].unwrapOr(null),
                    ]
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
                  [ReserveType.ReferendumInfo]: (deposit.unwrap
                    ? deposit.unwrap()
                    : deposit
                  ).amount.toBigInt(),
                });
              });
            });
          })
          .catch((error) => {
            console.error("Error fetching referendumInfoFor:", error);
            reject(error); // Reject the outer promise with the error
          });
      }
      resolve("referendumInfoFor scraped");
    });

    await new Promise((resolve, reject) => {
      apiAt.query.assets.asset
        .entries()
        .then(async (assets) => {
          assets.forEach((asset) => {
            updateReserveMap(asset[1].unwrap().owner.toHex().slice(-40), {
              [ReserveType.Asset]: asset[1].unwrap().deposit.toBigInt(),
            });
          });

          await new Promise((resolve, reject) => {
            apiAt.query.assets.metadata
              .entries()
              .then((assetsMetadata) => {
                assetsMetadata.forEach((assetMetadata) => {
                  updateReserveMap(
                    assets
                      .find(
                        (asset) =>
                          asset[0].toHex().slice(-64) == assetMetadata[0].toHex().slice(-64)
                      )[1]
                      .unwrap()
                      .owner.toHex()
                      .slice(-40),
                    {
                      [ReserveType.AssetMetadata]: assetMetadata[1].deposit.toBigInt(),
                    }
                  );
                });
                resolve("assetsMetadata scraped");
              })
              .catch((error) => {
                console.error("Error fetching assetsMetadata:", error);
                reject(error); // Reject the outer promise with the error
              });
          });

          resolve("assets scraped");
        })
        .catch((error) => {
          console.error("Error fetching assets :", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.localAssets.asset
        .entries()
        .then(async (localAssets) => {
          localAssets.forEach((localAsset) => {
            updateReserveMap(localAsset[1].unwrap().owner.toHex().slice(-40), {
              [ReserveType.LocalAsset]: localAsset[1].unwrap().deposit.toBigInt(),
            });
          });

          await new Promise((resolve, reject) => {
            apiAt.query.localAssets.metadata
              .entries()
              .then((localAssetMetadata) => {
                localAssetMetadata.forEach((localAssetMetadata) => {
                  updateReserveMap(
                    localAssets
                      .find(
                        (localAsset) =>
                          localAsset[0].toHex().slice(-64) ==
                          localAssetMetadata[0].toHex().slice(-64)
                      )[1]
                      .unwrap()
                      .owner.toHex()
                      .slice(-40),
                    {
                      [ReserveType.LocalAssetMetadata]: localAssetMetadata[1].deposit.toBigInt(),
                    }
                  );
                });
                resolve("localAssetsMetadata scraped");
              })
              .catch((error) => {
                console.error("Error fetching localAssetsMetadata:", error);
                reject(error); // Reject the outer promise with the error
              });
          });

          resolve("localAssets scraped");
        })
        .catch((error) => {
          console.error("Error fetching localAssets :", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.assetManager.localAssetDeposit
        .entries()
        .then((localAssetDeposits) => {
          localAssetDeposits.forEach((assetDeposit) => {
            updateReserveMap(assetDeposit[1].unwrap().creator.toHex(), {
              [ReserveType.LocalAssetDeposit]: assetDeposit[1].unwrap().deposit.toBigInt(),
            });
          });
          resolve("localAssetDeposits scraped");
        })
        .catch((error) => {
          console.error("Error fetching localAssetDeposits:", error);
          reject(error); // Reject the outer promise with the error
        });
    });

    await new Promise((resolve, reject) => {
      apiAt.query.balances.reserves
        .entries()
        .then((namedReserves) => {
          namedReserves.forEach((namedReservesOf) => {
            updateReserveMap(namedReservesOf[0].toHex().slice(-40), {
              [ReserveType.Named]: namedReservesOf[1]
                .map((namedDeposit) => namedDeposit.amount.toBigInt())
                .reduce((accumulator, curr) => accumulator + curr),
            });
          });
          resolve("namedReserves scraped");
        })
        .catch((error) => {
          console.error("Error fetching namedReserves:", error);
          reject(error); // Reject the outer promise with the error
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

    //1b) Build Expected Results - Locks Map

    await new Promise((resolve, reject) => {
      apiAt.query.balances.locks
        .entries()
        .then((locks) => {
          locks.forEach((lock) => {
            const key = hexToBase64(lock[0].toHex().slice(-40));
            const total = lock[1].reduce((acc, curr) => {
              return curr.amount.toBigInt() + acc;
            }, 0n);
            locksMap.set(key, { total });
          });
          resolve("locks scraped");
        })
        .catch((error) => {
          console.error("Error fetching locks:", error);
          reject(error);
        });
    });

    // Only applies to OpenGov
    if (
      (specVersion >= 1900 && runtimeName == "moonbase") ||
      (specVersion >= 2100 && runtimeName == "moonriver")
    ) {
      await new Promise((resolve, reject) => {
        apiAt.query.convictionVoting.votingFor
          .entries()
          .then(
            (votingFor: [StorageKey<[AccountId20, u16]>, PalletConvictionVotingVoteVoting][]) => {
              votingFor.forEach((votes) => {
                if (votes[1].isCasting) {
                  const accountId = votes[0].args[0].toHex().slice(-40);
                  const convictionVoting = votes[1].asCasting.votes.reduce((acc, curr) => {
                    const amount = curr[1].isStandard
                      ? curr[1].asStandard.balance.toBigInt()
                      : curr[1].isSplit
                      ? curr[1].asSplit.aye.toBigInt() + curr[1].asSplit.nay.toBigInt()
                      : curr[1].isSplitAbstain
                      ? curr[1].asSplitAbstain.aye.toBigInt() +
                        curr[1].asSplitAbstain.nay.toBigInt() +
                        curr[1].asSplitAbstain.abstain.toBigInt()
                      : 0n;

                    return acc > amount ? acc : amount;
                  }, 0n);
                  updateExpectedLocksMap(accountId, { convictionVoting });
                }
              });
              resolve("convictionVoting scraped");
            }
          )
          .catch((error) => {
            console.error("Error fetching convictionVoting:", error);
            reject(error);
          });
      });
    }

    delegatorState.forEach((delegator) => {
      if (
        specVersion < 1700 ||
        (specVersion < 1800 && !delegatorStakingMigrationAccounts[delegator[0].toHex().slice(-40)])
      ) {
        updateReserveMap(delegator[0].toHex().slice(-40), {
          [ReserveType.Delegator]: delegator[1].unwrap().total.toBigInt(),
        });
      }
    });

    delegatorState.forEach((delegator) => {
      // Support the case of the migration in 1700
      if (
        specVersion >= 1800 ||
        delegatorStakingMigrationAccounts[delegator[0].toHex().slice(-40)]
      ) {
        updateExpectedLocksMap(delegator[0].toHex().slice(-40), {
          DelStake: delegator[1].unwrap().total.toBigInt(),
        });
      }
    });

    democracyVotes.forEach((votes) => {
      if (votes[1].isDirect) {
        const accountId = votes[0].toHex().slice(-40);

        const democracy = votes[1].asDirect.votes.reduce((acc, curr) => {
          const subTotal = curr[1].isStandard
            ? curr[1].asStandard.balance.toBigInt()
            : curr[1].isSplit
            ? curr[1].asSplit.aye.toBigInt() + curr[1].asSplit.nay.toBigInt()
            : 0n;
          return acc > subTotal ? acc : subTotal;
        }, 0n);
        updateExpectedLocksMap(accountId, { democracy });
      }

      if (votes[1].isDelegating) {
        const accountId = votes[0].toHex().slice(-40);
        const delegatedDemocracy = votes[1].asDelegating.prior[1].toBigInt();
        updateExpectedLocksMap(accountId, { delegatedDemocracy });
      }
    });

    debug(`Retrieved ${expectedLocksMap.size} accounts with locks`);

    expectedLocksMap.forEach(({ locks }, key) => {
      const total = Object.values(locks).reduce((total, amount) => {
        total += amount;
        return total;
      }, 0n);
      expectedLocksMap.set(key, { locks, total });
    });

    //2) Build Actual Results - System Accounts
    const limit = 1000;
    let last_key = "";
    let count = 0;

    const checkReservedBalance = (userId: string, reservedBalance: bigint) => {
      const key = hexToBase64(userId);
      const expected = expectedReserveMap.has(key) ? expectedReserveMap.get(key).total : 0n;
      if (expected !== reservedBalance) {
        debug(`⚠️  Reserve balance mismatch for ${base64ToHex(key)}`);
        failedReserved.push(
          `⚠️  ${base64ToHex(key)} (reserved: ${reservedBalance} vs expected: ${expected})\n` +
            `\tℹ️  Expected contains: (${Object.keys(
              (expectedReserveMap.has(key) && expectedReserveMap.get(key).reserved) || {}
            )
              .map(
                (reserveType) =>
                  `${getReserveTypeByValue(reserveType)}:${printTokens(
                    context.polkadotApi,
                    expectedReserveMap.get(key).reserved[reserveType],
                    1,
                    5
                  )}`
              )
              .join(` - `)})`
        );
      }
      expectedReserveMap.delete(key);
    };

    if (process.env.ACCOUNT_ID) {
      const userId = process.env.ACCOUNT_ID;
      const user = await apiAt.query.system.account(userId);
      checkReservedBalance(userId, user.data.reserved.toBigInt());
    } else {
      // Loop over ALL System accounts
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
          const accountId = user[0].toHex().slice(-40);
          totalIssuance += user[1].data.free.toBigInt() + user[1].data.reserved.toBigInt();
          totalAccounts++;
          checkReservedBalance(accountId, user[1].data.reserved.toBigInt());
        }
        if (count % (10 * limit) == 0) {
          debug(`Checked ${count} accounts`);
        }
      }
      debug(`Checked ${totalAccounts} total accounts`);
    }

    //3) Collect and process locks failures
    // Loose check because we don't have a way to clever way to verify expired but unclaimed locks
    locksMap.forEach((value, key) => {
      if (expectedLocksMap.has(key)) {
        if (expectedLocksMap.get(key).total > value.total) {
          failedLocks.push(
            `\t${base64ToHex(key)} (total: actual ${printTokens(
              context.polkadotApi,
              value.total
            )} - expected: ${printTokens(context.polkadotApi, expectedLocksMap.get(key).total)})`
          );
        }
      }
    });
  });

  testIt("C100", `should have matching deposit/reserved`, async function () {
    if (failedReserved.length > 0) {
      debug("Failed accounts reserves");
    }

    expect(
      failedReserved.length,
      `❌ Mismatched account reserves: \n${failedReserved.join(",\n")}`
    ).to.equal(0);

    debug(`Verified ${totalAccounts} total reserve balances (at #${atBlockNumber})`);

    const failuresExpectedReserveMap = [];
    if (expectedReserveMap.size > 0) {
      debug(`expectedReserveMap size: ${expectedReserveMap.size}`);
      expectedReserveMap.forEach((value, key) => {
        failuresExpectedReserveMap.push(`${base64ToHex(key)}`);
      });
    }

    if (!process.env.ACCOUNT_ID) {
      expect(
        expectedReserveMap.size,
        `❌  There are accounts with expected reserve amounts not accounted for: ` +
          `${failuresExpectedReserveMap.join(`, `)}`
      ).to.equal(0);
    }
  });

  testIt("C200", "should match total locks", async function () {
    if (failedLocks.length > 0) {
      debug("Failed accounts locks");
    }
    expect(failedLocks.length, `❌  Failed accounts locks: \n${failedLocks.join(",\n")}`).to.equal(
      0
    );
  });

  testIt("C300", `should match total supply`, async function () {
    if (!!process.env.ACCOUNT_ID) {
      debug(`Env var ACCOUNT_ID set, skipping total supply check`);
      this.skip();
    }
    const queriedIssuance = (await apiAt.query.balances.totalIssuance()).toBigInt();

    debug(`Verified total issuance to be ${totalIssuance / 10n ** 18n}  ${symbol}`);
    expect(queriedIssuance).to.equal(totalIssuance);
  });
});
