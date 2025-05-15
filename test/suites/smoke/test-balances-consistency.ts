import "@moonbeam-network/api-augment/moonbase";
import type { ApiDecoration } from "@polkadot/api/types";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { hexToBigInt, u8aConcat, u8aToHex } from "@polkadot/util";
import type { u16 } from "@polkadot/types";
import type { AccountId20 } from "@polkadot/types/interfaces";
import type {
  PalletReferendaDeposit,
  PalletConvictionVotingVoteVoting,
} from "@polkadot/types/lookup";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { TWO_HOURS, printTokens } from "@moonwall/util";
import type { StorageKey } from "@polkadot/types";
import { extractPreimageDeposit, AccountShortfalls } from "../../helpers";
import type { ApiPromise } from "@polkadot/api";
import { processAllStorage } from "../../helpers/storageQueries.js";

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
  MultiSig = "20",
  PreimageBalanceHolds = "21",
}

type ReservedInfo = { total?: bigint; reserved?: { [key: string]: bigint } };
type LocksInfo = { total?: bigint; locks?: { [key: string]: bigint } };

// This test attemps to reconcile the total amount of locked tokens across the entire
// chain by ensuring that individual storages match the reserved balances and locks.
// In order to not exhaust memory, the expected results are calculated first and then
// All system accounts are iterated over, without storing them, to ensure memory
// is not exhausted.

describeSuite({
  id: "S03",
  title: "Verifying balances consistency",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    const expectedReserveMap = new Map<string, ReservedInfo>();
    const expectedLocksMap = new Map<string, LocksInfo>();
    const locksMap = new Map<string, { total: bigint }>();
    const failedLocks: any[] = [];
    const failedReserved: any[] = [];
    let atBlockNumber = 0;
    let apiAt: ApiDecoration<"promise">;
    let specVersion = 0;
    let runtimeName: string;
    let totalAccounts = 0n;
    let totalIssuance = 0n;
    let symbol: string;
    let paraApi: ApiPromise;

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

      let tempRegister = {};
      Object.keys(newReserve).forEach((key) => {
        if (value.reserved![key]) {
          tempRegister = { [key]: value.reserved![key] + newReserve[key] };
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
        if (ReserveType[key as keyof typeof ReserveType] === value) {
          return key;
        }
      }
      return null;
    };

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const blockHash = process.env.BLOCK_NUMBER
        ? (await paraApi.rpc.chain.getBlockHash(Number.parseInt(process.env.BLOCK_NUMBER))).toHex()
        : (await paraApi.rpc.chain.getFinalizedHead()).toHex();
      atBlockNumber = (await paraApi.rpc.chain.getHeader(blockHash)).number.toNumber();
      apiAt = await paraApi.at(blockHash);
      specVersion = apiAt.consts.system.version.specVersion.toNumber();
      runtimeName = apiAt.runtimeVersion.specName.toString();
      symbol = (await paraApi.rpc.system.properties()).tokenSymbol.unwrap()[0].toString();

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
            reject(error);
          });
      });

      const [
        democracyVotes,
        delegatorState,
        delegatorStakingMigrations,
        collatorStakingMigrations,
      ] = await Promise.all([
        specVersion < 2900 ? apiAt.query.democracy.votingOf.entries() : undefined,
        apiAt.query.parachainStaking.delegatorState.entries(),
        specVersion >= 1700 && specVersion < 1800
          ? apiAt.query.parachainStaking.delegatorReserveToLockMigrations.entries()
          : undefined,
        specVersion >= 1700 && specVersion < 1800
          ? apiAt.query.parachainStaking.collatorReserveToLockMigrations.entries()
          : undefined,
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
            reject(error);
          });
      });

      const delegatorStakingMigrationAccounts =
        delegatorStakingMigrations !== undefined
          ? (delegatorStakingMigrations.reduce((p, migration: any) => {
              if (migration[1].isTrue) {
                p[`0x${migration[0].toHex().slice(-40)}`] = true;
              }
              return p;
            }, {} as any) as { [account: string]: boolean })
          : {};

      const collatorStakingMigrationAccounts =
        collatorStakingMigrations !== undefined
          ? (collatorStakingMigrations.reduce((p, migration: any) => {
              if (migration[1].isTrue) {
                p[`0x${migration[0].toHex().slice(-40)}`] = true;
              }
              return p;
            }, {} as any) as { [account: string]: boolean })
          : {};

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
            reject(error);
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
            reject(error);
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
            reject(error);
          });
      });

      await new Promise((resolve, reject) => {
        apiAt.query.identity.identityOf
          .entries()
          .then((identities) => {
            identities.forEach((identity) => {
              const storageValue = (() => {
                if (specVersion < 2900) {
                  return identity[1].unwrap();
                } else if (specVersion < 3700) {
                  return identity[1].unwrap()[0];
                }
                return identity[1].unwrap();
              })();
              updateReserveMap(identity[0].toHex().slice(-40), {
                [ReserveType.Identity]: storageValue.deposit.toBigInt(),
              });
              updateReserveMap(identity[0].toHex().slice(-40), {
                [ReserveType.RequestJudgements]: storageValue.judgements.reduce(
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
            reject(error);
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
            reject(error);
          });
      });

      await new Promise((resolve, reject) => {
        if (specVersion < 2900) {
          apiAt.query.democracy.depositOf
            .entries()
            .then((democracyDeposits) => {
              democracyDeposits
                .map((depositOf) =>
                  depositOf[1]
                    .unwrap()[0]
                    .flatMap((deposit) => ({
                      accountId: deposit.toHex(),
                      reserved: depositOf[1].unwrap()[1].toBigInt(),
                    }))
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
                        p[deposit.accountId].reserved[ReserveType.DemocracyDeposit] +=
                          deposit.reserved;
                        return p;
                      },
                      {} as {
                        [account: string]: {
                          accountId: `0x${string}`;
                          reserved: {
                            [ReserveType.DemocracyDeposit]: bigint;
                          };
                        };
                      }
                    )
                )
                .forEach((deposit: any) => {
                  updateReserveMap(deposit.accountId, deposit.reserved);
                });

              Object.values(
                democracyDeposits
                  .flatMap((depositOf) =>
                    depositOf[1].unwrap()[0].map((deposit) => ({
                      accountId: deposit.toHex(),
                      reserved: depositOf[1].unwrap()[1].toBigInt(),
                    }))
                  )
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
                      p[deposit.accountId].reserved[ReserveType.DemocracyDeposit] +=
                        deposit.reserved;
                      return p;
                    },
                    {} as {
                      [accountId: string]: {
                        accountId: string;
                        reserved: { [key: string]: bigint };
                      };
                    }
                  )
              ).forEach((deposit: any) => {
                updateReserveMap(deposit.accountId, deposit.reserved);
              });
            })
            .catch((error) => {
              console.error("Error fetching democracy deposits:", error);
              reject(error);
            });
        }
        resolve("democracy deposits scraped");
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
              reject(error);
            });
        }
        resolve("democracyPreimages scraped");
      });

      await new Promise((resolve, reject) => {
        if ((specVersion >= 1900 && runtimeName === "moonbase") || specVersion >= 2000) {
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
                  return deposit
                    ? { accountId: deposit.accountId, deposit: deposit.amount }
                    : undefined;
                })
                .filter((value) => typeof value !== "undefined")
                .forEach(({ deposit, accountId }: any) => {
                  updateReserveMap(accountId, {
                    [ReserveType.PreimageStatus]: deposit === 0n ? 0n : deposit.toBigInt(),
                  });
                });
            })
            .catch((error) => {
              console.error("Error fetching proxies:", error);
              reject(error);
            });
          resolve("proxies scraped");
        }
      });

      await new Promise((resolve, reject) => {
        if (
          (specVersion >= 1900 && runtimeName === "moonbase") ||
          (specVersion >= 2100 && runtimeName === "moonriver") ||
          (specVersion >= 2300 && runtimeName === "moonbeam")
        ) {
          apiAt.query.referenda.referendumInfoFor
            .entries()
            .then((referendumInfoFor) => {
              referendumInfoFor.forEach((info) => {
                const deposits = (
                  info[1].unwrap().isApproved
                    ? [
                        info[1].unwrap().asApproved[1].unwrapOr(null),
                        info[1].unwrap().asApproved[2].unwrapOr(null),
                      ]
                    : info[1].unwrap().isRejected
                      ? [
                          info[1].unwrap().asRejected[1].unwrapOr(null),
                          info[1].unwrap().asRejected[2].unwrapOr(null),
                        ]
                      : info[1].unwrap().isCancelled
                        ? [
                            info[1].unwrap().asCancelled[1].unwrapOr(null),
                            info[1].unwrap().asCancelled[2].unwrapOr(null),
                          ]
                        : info[1].unwrap().isTimedOut
                          ? [
                              info[1].unwrap().asTimedOut[1].unwrapOr(null),
                              info[1].unwrap().asTimedOut[2].unwrapOr(null),
                            ]
                          : info[1].unwrap().isOngoing
                            ? [
                                info[1].unwrap().asOngoing.submissionDeposit,
                                info[1].unwrap().asOngoing.decisionDeposit.unwrapOr(null),
                              ]
                            : ([] as PalletReferendaDeposit[])
                ).filter((value) => !!value);

                deposits.forEach((deposit) => {
                  // Support for https://github.com/paritytech/substrate/pull/12788
                  // which make deposit optional.
                  // TODO: better handle unwrapping
                  updateReserveMap(deposit!.who.toHex(), {
                    [ReserveType.ReferendumInfo]: deposit!.amount.toBigInt(),
                  });
                });
              });
            })
            .catch((error) => {
              console.error("Error fetching referendumInfoFor:", error);
              reject(error);
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
                            asset[0].toHex().slice(-64) === assetMetadata[0].toHex().slice(-64)
                        )![1]
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
                  reject(error);
                });
            });

            resolve("assets scraped");
          })
          .catch((error) => {
            console.error("Error fetching assets :", error);
            reject(error);
          });
      });

      if (specVersion < 2800) {
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
                              localAsset[0].toHex().slice(-64) ===
                              localAssetMetadata[0].toHex().slice(-64)
                          )![1]
                          .unwrap()
                          .owner.toHex()
                          .slice(-40),
                        {
                          [ReserveType.LocalAssetMetadata]:
                            localAssetMetadata[1].deposit.toBigInt(),
                        }
                      );
                    });
                    resolve("localAssetsMetadata scraped");
                  })
                  .catch((error) => {
                    console.error("Error fetching localAssetsMetadata:", error);
                    reject(error);
                  });
              });

              resolve("localAssets scraped");
            })
            .catch((error) => {
              console.error("Error fetching localAssets :", error);
              reject(error);
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
              reject(error);
            });
        });
      }

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
            reject(error);
          });
      });

      if (specVersion >= 2700) {
        await new Promise((resolve, reject) => {
          apiAt.query.balances.holds
            .entries()
            .then((holds) => {
              holds.forEach((holdsOf) => {
                const accountId = holdsOf[0].toHex().slice(-40);
                holdsOf[1].forEach((holdOf) => {
                  if (holdOf.id.isPreimage) {
                    updateReserveMap(accountId, {
                      [ReserveType.PreimageBalanceHolds]: holdOf.amount.toBigInt(),
                    });
                  } else {
                    throw `Unknown hold id ${holdOf.id}`;
                  }
                });
              });
              resolve("Preimage balance hold scraped");
            })
            .catch((error) => {
              console.error("Error fetching holds:", error);
              reject(error);
            });
        });
      }

      if (specVersion >= 2401) {
        await new Promise((resolve, reject) => {
          apiAt.query.multisig.multisigs
            .entries()
            .then((multisigs) => {
              multisigs.forEach((multisig) => {
                const json = (multisig[1] as any).toJSON();
                updateReserveMap(json.depositor, {
                  [ReserveType.MultiSig]: BigInt(json.deposit),
                });
              });
              resolve("multiSigs scraped");
            })
            .catch((error) => {
              console.error("Error fetching multisigs:", error);
              reject(error);
            });
        });
      }

      log(`Retrieved ${expectedReserveMap.size} deposits`);
      expectedReserveMap.forEach(({ reserved }, key) => {
        const total = Object.values(reserved!).reduce((total, amount) => {
          const subtotal = total + amount;
          return subtotal;
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
        (specVersion >= 1900 && runtimeName === "moonbase") ||
        (specVersion >= 2100 && runtimeName === "moonriver") ||
        (specVersion >= 2300 && runtimeName === "moonbeam")
      ) {
        await new Promise((resolve, reject) => {
          apiAt.query.convictionVoting.votingFor
            .entries()
            .then((votingFor) => {
              (
                votingFor as [StorageKey<[AccountId20, u16]>, PalletConvictionVotingVoteVoting][]
              ).forEach((votes) => {
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
            })
            .catch((error) => {
              console.error("Error fetching convictionVoting:", error);
              reject(error);
            });
        });
      }

      delegatorState.forEach((delegator) => {
        if (
          specVersion < 1700 ||
          (specVersion < 1800 &&
            !delegatorStakingMigrationAccounts[delegator[0].toHex().slice(-40)])
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

      if (specVersion < 2900) {
        if (!democracyVotes) {
          throw new Error("Democracy votes not found");
        }

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
      }

      log(`Retrieved ${expectedLocksMap.size} accounts with locks`);

      expectedLocksMap.forEach(({ locks }, key) => {
        const total = Object.values(locks!).reduce((total, amount) => {
          const subtotal = total + amount;
          return subtotal;
        }, 0n);
        expectedLocksMap.set(key, { locks, total });
      });

      ///
      //2) Build Actual Results - System Accounts
      ///

      // This code block queries and processes storage keys and values for System.Accounts via
      // manual RPC methods. It uses pagination to efficiently query keys, measures performance and
      // memory usage, and estimates the remaining time. Once all keys are fetched, it processes
      // the account information, updating total issuance and total accounts. It also checks the
      // reserved balance for each account and logs memory usage and performance metrics. The code
      // is organized into two main sections:
      //  -  Querying and storing System.Account storage keys.
      //  -  Processing the account information by querying storage values for each key,
      //     calculating total issuance, total accounts, and checking reserved balances.

      // Example Manual Decode
      // Key: 0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9
      //        00e8c90d5df372b81979d8930f4586f511e743c2fe35f30d4c7dda982109376fc6d76410
      //   - last 20 bytes is the account id (11e743c2fe35f30d4c7dda982109376fc6d76410)
      //
      // Value: 0x000000000000000001000000000000000000dc0958f8871e00000000000000000000000000
      // 00000000000000000000000000000000000000000000000000000000000000000000000000000000000000

      // AccountInfo = Struct({
      //   nonce: u32,                    (00000000)
      //   consumers: u32,                (00000000)
      //   providers: u32,                (01000000)
      //   sufficients: u32,              (00000000)
      //   data: Struct({
      //     free: u128,                  (0000dc0958f8871e0000000000000000)
      //     reserved: u128,              (00000000000000000000000000000000)
      //     miscFrozen: u128,            (00000000000000000000000000000000)
      //     feeFrozen: u128,             (00000000000000000000000000000000)
      //   }),
      // });

      const checkReservedBalance = (userId: string, reservedBalance: bigint) => {
        const key = hexToBase64(userId);
        const expected = expectedReserveMap.has(key) ? expectedReserveMap.get(key)!.total : 0n;
        if (expected !== reservedBalance) {
          log(`⚠️  Reserve balance mismatch for ${base64ToHex(key)}`);
          // Editor Config doesn't like this string hence the insane fragmentation
          const errorString =
            `⚠️  ${base64ToHex(key)} (reserved: ${reservedBalance}` +
            ` vs expected: ${expected})\n` +
            "\tℹ️  Expected contains: (" +
            Object.keys(
              (expectedReserveMap.has(key) && expectedReserveMap.get(key)!.reserved) || {}
            )
              .map(
                (reserveType) =>
                  getReserveTypeByValue(reserveType) +
                  ":" +
                  printTokens(paraApi, expectedReserveMap.get(key)!.reserved![reserveType], 1, 5)
              )
              .join(` - `) +
            `)`;
          failedReserved.push(errorString);
        }
        expectedReserveMap.delete(key);
      };

      const keyPrefix = u8aToHex(
        u8aConcat(xxhashAsU8a("System", 128), xxhashAsU8a("Account", 128))
      );

      const t0 = performance.now();
      if (process.env.ACCOUNT_ID) {
        const userId = process.env.ACCOUNT_ID;
        const user = await apiAt.query.system.account(userId);
        checkReservedBalance(userId, user.data.reserved.toBigInt());
        totalAccounts++;
      } else {
        const chainName = (await paraApi.rpc.system.chain()).toString();
        const shortFallExists = AccountShortfalls[chainName];
        const runtimeVersion = paraApi.consts.system.version.specVersion.toNumber();

        await processAllStorage(paraApi, keyPrefix, blockHash, (items) => {
          for (const { key, value } of items) {
            const accountId = key.slice(-40);
            const accountInfo = value;
            const freeBal = hexToBigInt(accountInfo.slice(34, 66), { isLe: true });
            let reservedBalance = hexToBigInt(accountInfo.slice(66, 98), { isLe: true });
            totalIssuance += freeBal + reservedBalance;
            totalAccounts++;

            if (
              shortFallExists?.[`0x${accountId}`] &&
              shortFallExists[`0x${accountId}`].brokenIn <= runtimeVersion
            ) {
              reservedBalance -= shortFallExists[`0x${accountId}`].reserved;
            }

            checkReservedBalance(accountId, reservedBalance);
          }
        });
        const t1 = performance.now();
        const checkTime = (t1 - t0) / 1000;
        const text =
          checkTime < 60
            ? `${checkTime.toFixed(1)} seconds`
            : `${(checkTime / 60).toFixed(1)} minutes`;
        log(`Finished checking ${totalAccounts} System.Account storage values in ${text} ✅`);
      }

      //3) Collect and process locks failures
      // Loose check because we don't have a clever way to verify expired but unclaimed locks
      locksMap.forEach((value, key) => {
        if (expectedLocksMap.has(key)) {
          if (expectedLocksMap.get(key)!.total! > value.total) {
            failedLocks.push(
              `\t${base64ToHex(key)} (total: actual ${printTokens(
                paraApi,
                value.total
              )} - expected: ${printTokens(paraApi, expectedLocksMap.get(key)!.total!)})`
            );
          }
        }
      });
    }, TWO_HOURS);

    it({
      id: "C100",
      title: "should have matching deposit/reserved",
      test: async function () {
        if (failedReserved.length > 0) {
          log("Failed accounts reserves");
        }

        expect(
          failedReserved.length,
          `❌ Mismatched account reserves: \n${failedReserved.join(",\n")}`
        ).to.equal(0);

        log(`Verified ${totalAccounts} total reserve balances (at #${atBlockNumber})`);

        const failuresExpectedReserveMap: string[] = [];
        if (expectedReserveMap.size > 0) {
          log(`expectedReserveMap size: ${expectedReserveMap.size}`);
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
      },
    });

    it({
      id: "C200",
      title: "should match total locks",
      test: async function () {
        if (failedLocks.length > 0) {
          log("Failed accounts locks");
        }
        expect(
          failedLocks.length,
          `❌  Failed accounts locks: \n${failedLocks.join(",\n")}`
        ).to.equal(0);
      },
    });

    it({
      id: "C300",
      title: "should match total supply",
      test: async function () {
        if (process.env.ACCOUNT_ID) {
          log(`Env var ACCOUNT_ID set, skipping total supply check`);
          return;
        }
        const queriedIssuance = (await apiAt.query.balances.totalIssuance()).toBigInt();

        log(`Verified total issuance to be ${totalIssuance / 10n ** 18n}  ${symbol}`);
        expect(queriedIssuance).to.equal(totalIssuance);
      },
    });
  },
});
