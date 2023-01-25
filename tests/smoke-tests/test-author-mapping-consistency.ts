import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:author-mapping");

describeSmokeSuite("S200", `Verifying deposit for associated nimbus ids`, (context, testIt) => {
  const nimbusIdPerAccount: { [account: string]: string } = {};

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;

  before("Retrieve all associated nimbus ids", async function () {
    // It takes time to load all the nimbus ids.
    this.timeout(300_000); // 5min

    // How many entries to query at a time
    const limit = 1000;

    // Last key to query in a loop
    let last_key = "";

    // current number of queried items
    let count = 0;

    // Configure the api at a specific block
    // (to avoid inconsistency querying over multiple block when the test takes a long time to
    // query data and blocks are being produced)
    atBlockNumber = process.env.BLOCK_NUMBER
      ? parseInt(process.env.BLOCK_NUMBER)
      : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );

    // Query nimbus ids
    while (true) {
      const query = await apiAt.query.authorMapping.nimbusLookup.entriesPaged({
        args: [],
        pageSize: limit,
        startKey: last_key,
      });

      if (query.length == 0) {
        break;
      }
      count += query.length;

      // Convert data to a dictonnary accountId -> nimbusId
      for (const entry of query) {
        const accountId = `0x${entry[0].toHex().slice(-40)}`;
        last_key = entry[0].toString();
        nimbusIdPerAccount[accountId] = entry[1].toString();
      }

      // Debug logs to make sure it keeps progressing
      if (count % (10 * limit) == 0) {
        debug(`Retrieved ${count} nimbus ids`);
      }
    }

    debug(`Retrieved ${count} total nimbus ids`);
  });

  testIt("C100", `should have a deposit for each associated nimbus id`, async function () {
    this.timeout(60_000);

    // Instead of putting an expect in the loop. We track all failed entries instead
    const failedEntries: { accountId: string; nimbusId: string; problem: string }[] = [];

    // Verify that there is a deposit for each nimbus id
    for (const accountId of Object.keys(nimbusIdPerAccount)) {
      const nimbusId = nimbusIdPerAccount[accountId];
      const registrationInfo = await apiAt.query.authorMapping.mappingWithDeposit(nimbusId);
      if (registrationInfo.isNone || registrationInfo.unwrap().deposit.toBigInt() <= BigInt(0)) {
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
        return prev + (c == "0" ? 1 : 0);
      }, 0);
      if (zeroes > 32) {
        // this isn't an inconsistent state, so we will just warn.
        //
        // we could also check whether this account exists as a collator candidate, as the
        // combination of bogus keys and being an eligible author would mean the candidate could
        // never produce a block when `pallet_randomness` is enabled for the runtime
        console.log(`Warning: AuthorMapping ${accountId} exists with suspicious keys: ${keys_}`);
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
    debug(
      `Verified ${Object.keys(nimbusIdPerAccount).length} total accounts (at #${atBlockNumber})`
    );
  });
});
