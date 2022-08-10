import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:ethereum-contract");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite( `Ethereum contract bytecode should not be large`,
  { wssUrl, relayWssUrl },
  (context) => {

    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;

    const accountCodes: { [account: string]: string } = {};

    before("Retrieve all contrcact bytecode", async function () {
      this.timeout(7_200_000);

      const limit = 1000;
      let last_key = "";
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

      let timer;
      let doOneRequest = async () => {
        let query = await apiAt.query.evm.accountCodes.entriesPaged({
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
          accountCodes[accountId] = accountCode[1].toHex();
        }

        // Debug logs to make sure it keeps progressing
        if (count % (10 * limit) == 0) {
          debug(`Retrieved ${count} accountCodes`);
        }

        return false;
      };

      await new Promise<void>((resolve) => {
        const run = async () => {
          let done = await doOneRequest();
          if (done) {
            resolve();
          } else {
            setTimeout(run, 100);
          }
        };

        setTimeout(run, 100);
      });

      debug(`Retrieved ${count} total accountCodes`);
    });

    it("should not have excessively long account codes", async function () {
      this.timeout(300_000);

      // taken from geth, e.g. search "MaxCodeSize":
      // https://github.com/etclabscore/core-geth/blob/master/params/vars/protocol_params.go
      const MAX_CONTRACT_SIZE_BYTES = 24576;
      const MAX_CONTRACT_SIZE_HEX = 2 + 2 * MAX_CONTRACT_SIZE_BYTES;

      const failedContractCodes: { accountId: string; codesize: number }[] = [];

      const hexSizeToByteSize = (hexSize: number): number => {
        return hexSize / 2 - 2;
      }

      for (const accountId of Object.keys(accountCodes)) {
        const contractCode = accountCodes[accountId];
        if (contractCode.length > MAX_CONTRACT_SIZE_HEX) {
          failedContractCodes.push({ accountId, codesize: hexSizeToByteSize(contractCode.length) });
        }
      }

      console.log("Failed account codes (too long):");
      console.log(
        failedContractCodes
          .map(({ accountId, codesize }) => {
            return `accountId: ${accountId} - ${chalk.red(codesize)} bytes`;
          })
          .join(`\n`)
      );

      // Make sure the test fails after we print the errors
      expect(failedContractCodes.length, "Failed contract code max length").to.equal(0);

      // Additional debug logs
      debug(
        `Verified ${Object.keys(accountCodes).length} total account codes (at #${atBlockNumber})`
      );
    });
  }
);
