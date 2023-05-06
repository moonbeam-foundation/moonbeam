import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { THIRTY_MINS } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";

// TODO: Once balanceConsistency refactor PR merged, update this tc to use the new fast query logic
describeSuite({
  id: "S600",
  title: `Ethereum contract bytecode should not be large`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;
    const accountCodeSizesByAddress: { [account: string]: number } = {};
    let paraApi: ApiPromise;

    // returns the length in bytes of the byte array represented by the given hex string.
    // assumes a prefixed "0x".
    const byteLengthOfHexString = (hex: string): number => {
      return (hex.length - 2) / 2;
    };

    beforeAll(async function () {
      paraApi = context.polkadotJs({ apiName: "para" });
      const limit = 500;
      let last_key = "";
      let count = 0;

      // Configure the api at a specific block
      // (to avoid inconsistency querying over multiple block when the test takes a long time to
      // query data and blocks are being produced)
      atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));

      const doOneRequest = async () => {
        const query = await apiAt.query.evm.accountCodes.entriesPaged({
          args: [],
          pageSize: limit,
          startKey: last_key,
        });

        if (query.length == 0) {
          return true;
        }
        count += query.length;

        for (const accountCode of query) {
          let accountId = `0x${accountCode[0].toHex().slice(-40)}`;
          last_key = accountCode[0].toString();
          accountCodeSizesByAddress[accountId] = byteLengthOfHexString(accountCode[1].toHex());
        }

        // Debug logs to make sure it keeps progressing
        if (count % (10 * limit) == 0) {
          log(`Retrieved ${count} accountCodes`);
        }

        return false;
      };

      await new Promise<void>((resolve) => {
        const run = async () => {
          const done = await doOneRequest();
          if (done) {
            resolve();
          } else {
            setTimeout(run, 100);
          }
        };

        setTimeout(run, 100);
      });

      log(`Retrieved ${count} total accountCodes`);
    }, THIRTY_MINS);

    it({
      id: "C100",
      title: "should not have excessively long account codes",
      test: async function () {
        // taken from geth, e.g. search "MaxCodeSize":
        // https://github.com/etclabscore/core-geth/blob/master/params/vars/protocol_params.go
        const MAX_CONTRACT_SIZE_BYTES = 24576;
        const MAX_CONTRACT_SIZE_HEX = 2 + 2 * MAX_CONTRACT_SIZE_BYTES;
        const failedContractCodes: { accountId: string; codesize: number }[] = [];

        for (const accountId of Object.keys(accountCodeSizesByAddress)) {
          const codesize = accountCodeSizesByAddress[accountId];
          if (codesize > MAX_CONTRACT_SIZE_HEX) {
            failedContractCodes.push({ accountId, codesize });
          }
        }

        expect(
          failedContractCodes.length,
          `Failed account codes (too long): ${failedContractCodes
            .map(
              ({ accountId, codesize }) => `accountId: ${accountId} - ${chalk.red(codesize)} bytes`
            )
            .join(`, `)}`
        ).to.equal(0);

        const numAccounts = Object.keys(accountCodeSizesByAddress).length;
        log(`Verified ${numAccounts} total account codes (at #${atBlockNumber})`);
      },
    });
  },
});
