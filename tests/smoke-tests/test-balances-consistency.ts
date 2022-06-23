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
      subItentities,
      democracyDeposits,
      preimages,
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
      apiAt.query.parachainStaking.candidateInfo.entries(),
      apiAt.query.parachainStaking.delegatorState.entries(),
      apiAt.query.identity.identityOf.entries(),
      apiAt.query.identity.subsOf.entries(),
      apiAt.query.democracy.depositOf.entries(),
      apiAt.query.democracy.preimages.entries(),
      apiAt.query.assets.asset.entries(),
      apiAt.query.assets.metadata.entries(),
      apiAt.query.localAssets.asset.entries(),
      apiAt.query.localAssets.metadata.entries(),
      apiAt.query.assetManager.localAssetDeposit.entries(),
      apiAt.query.balances.reserves.entries(),
    ]);

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
      candidateInfo.map((candidate) => ({
        accountId: `0x${candidate[0].toHex().slice(-40)}`,
        reserved: {
          candidate: candidate[1].unwrap().bond.toBigInt(),
        },
      })),
      delegatorState.map((delegator) => ({
        accountId: `0x${delegator[0].toHex().slice(-40)}`,
        reserved: {
          delegator: delegator[1].unwrap().total.toBigInt(),
        },
      })),
      identities.map((identity) => ({
        accountId: `0x${identity[0].toHex().slice(-40)}`,
        reserved: {
          identity: identity[1].unwrap().deposit.toBigInt(),
        },
      })),
      subItentities.map((subIdentity) => ({
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

    const failedExpectations = [];

    for (const accountId of Object.keys(accounts)) {
      let reserved = accounts[accountId].data.reserved.toBigInt();
      const expectedReserve = expectedReserveByAccount[accountId]?.total || 0n;

      if (reserved != expectedReserve) {
        failedExpectations.push(
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

    if (failedExpectations.length > 0) {
      console.log(chalk.red(failedExpectations.join("\n")));
      expect(failedExpectations.length, "Failed accounts reserves").to.equal(0);
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
