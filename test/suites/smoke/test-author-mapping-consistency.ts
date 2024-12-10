import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { FIVE_MINS } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import type { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";

describeSuite({
  id: "S02",
  title: `Verifying deposit for associated nimbus ids`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    const nimbusIdPerAccount: { [account: string]: string } = {};

    let atBlockNumber = 0;
    let apiAt: ApiDecoration<"promise">;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const limit = 1000;
      let last_key = "";
      let count = 0;

      // Configure the api at a specific block
      // (to avoid inconsistency querying over multiple block when the test takes a long time to
      // query data and blocks are being produced)
      atBlockNumber = process.env.BLOCK_NUMBER
        ? Number.parseInt(process.env.BLOCK_NUMBER)
        : (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));

      // Query nimbus ids
      for (;;) {
        const query = await apiAt.query.authorMapping.nimbusLookup.entriesPaged({
          args: [],
          pageSize: limit,
          startKey: last_key,
        });

        if (query.length === 0) {
          break;
        }
        count += query.length;

        // Convert data to a dictionary accountId -> nimbusId
        for (const entry of query) {
          const accountId = `0x${entry[0].toHex().slice(-40)}`;
          last_key = entry[0].toString();
          nimbusIdPerAccount[accountId] = entry[1].toString();
        }

        // Debug logs to make sure it keeps progressing
        if (count % (10 * limit) === 0) {
          log(`Retrieved ${count} nimbus ids`);
        }
      }

      log(`Retrieved ${count} total nimbus ids`);
    }, FIVE_MINS);

    it({
      id: "C100",
      title: `should have a deposit for each associated nimbus id`,
      timeout: FIVE_MINS,
      test: async function () {
        const failedEntries: { accountId: string; nimbusId: string; problem: string }[] = [];

        // Verify that there is a deposit for each nimbus id
        for (const accountId of Object.keys(nimbusIdPerAccount)) {
          const nimbusId = nimbusIdPerAccount[accountId];
          const registrationInfo = await apiAt.query.authorMapping.mappingWithDeposit(nimbusId);

          if (
            registrationInfo.isNone ||
            registrationInfo.unwrap().deposit.toBigInt() <= BigInt(0)
          ) {
            failedEntries.push({ accountId, nimbusId, problem: "missing deposit" });
          }

          // ensure each account has reserve >= deposit
          const account = await apiAt.query.system.account(accountId);
          if (registrationInfo.unwrap().deposit.toBigInt() > account.data.reserved.toBigInt()) {
            failedEntries.push({ accountId, nimbusId, problem: "insufficient reserved amount" });
          }

          // ensure that keys exist and smell legitimate
          const keys_ = registrationInfo.unwrap().keys_;
          const zeroes = Array.from(keys_.toString()).reduce((prev, c) => {
            return prev + (c === "0" ? 1 : 0);
          }, 0);
          if (zeroes > 32) {
            // this isn't an inconsistent state, so we will just warn.
            //
            // we could also check whether this account exists as a collator candidate, as the
            // combination of bogus keys and being an eligible author would mean the candidate
            // could never produce a block when `pallet_randomness` is enabled for the runtime
            log(
              `${chalk.bgWhiteBright.blackBright(
                "Warning"
              )}: AuthorMapping ${accountId} exists with suspicious keys: ${keys_}`
            );
          }
        }

        expect(
          failedEntries.length,
          `Failed accounts without deposit: ${failedEntries
            .map(({ accountId, problem }) => {
              return `accountId: ${accountId}, problem: ${problem}`;
            })
            .join(`\n`)} `
        ).to.equal(0);
        log(
          `Verified ${Object.keys(nimbusIdPerAccount).length} total accounts (at #${atBlockNumber})`
        );
      },
    });
  },
});
