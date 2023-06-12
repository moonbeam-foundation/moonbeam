import "@moonbeam-network/api-augment";
import chalk from "chalk";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { rateLimiter } from "../util/common";

const debug = require("debug")("smoke:ethereum-contract");
const limiter = rateLimiter();

describeSmokeSuite("S600", `Ethereum contract bytecode should not be large`, (context, testIt) => {
  let atBlockNumber: number;
  let totalContracts: bigint = 0n;
  const failedContractCodes: { accountId: string; codesize: number }[] = [];

  before("Retrieve all contract bytecode", async function () {
    this.timeout(6_000_000); // 30 minutes

    const blockHash = process.env.BLOCK_NUMBER
      ? (
          await context.polkadotApi.rpc.chain.getBlockHash(parseInt(process.env.BLOCK_NUMBER))
        ).toHex()
      : (await context.polkadotApi.rpc.chain.getFinalizedHead()).toHex();
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader(blockHash)).number.toNumber();

    // taken from geth, e.g. search "MaxCodeSize":
    // https://github.com/etclabscore/core-geth/blob/master/params/vars/protocol_params.go
    const MAX_CONTRACT_SIZE_BYTES = 24576;
    const getBytecodeSize = (bytecode: string) => Math.ceil((bytecode.length - 2) / 2);

    // Max RPC response limit is 15728640 bytes (15MB), so pessimistically the pageLimit
    // needs to be lower than if every contract was above the MAX_CONTRACT_SIZE
    const limit = 500;
    const keyPrefix = u8aToHex(
      u8aConcat(xxhashAsU8a("EVM", 128), xxhashAsU8a("AccountCodes", 128))
    );
    const growthFactor = 1.5;
    let last_key = keyPrefix;
    let count = 0;
    let loggingFrequency = 10;
    let loopCount = 0;

    let pagedKeys = [];

    let t0 = performance.now();
    let t1 = t0;
    keys: while (true) {
      const queryResults = (
        await limiter.schedule(() =>
          context.polkadotApi.rpc.state.getKeysPaged(keyPrefix, limit, last_key, blockHash)
        )
      )
        .map((key) => key.toHex())
        .filter((key) => key.includes(keyPrefix));
      pagedKeys.push(...queryResults);
      count += queryResults.length;

      if (queryResults.length === 0) {
        break keys;
      }

      last_key = queryResults[queryResults.length - 1];

      if (count % (limit * loggingFrequency) == 0) {
        loopCount++;
        const t2 = performance.now();
        const duration = t2 - t1;
        const qps = (limit * loggingFrequency) / (duration / 1000);
        const used = process.memoryUsage().heapUsed / 1024 / 1024;
        debug(
          `Queried ${count} keys @ ${qps.toFixed(0)} keys/sec, ${used.toFixed(0)} MB heap used`
        );

        // Increase logging threshold after 5 prints
        if (loopCount % 5 === 0) {
          loggingFrequency = Math.floor(loggingFrequency ** growthFactor);
          debug(`⏫  Increased logging threshold to every ${loggingFrequency * limit} accounts`);
        }
      }
    }

    let t3 = performance.now();
    const keyQueryTime = (t3 - t0) / 1000;
    const keyText =
      keyQueryTime > 60
        ? `${(keyQueryTime / 60).toFixed(1)} minutes`
        : `${keyQueryTime.toFixed(1)} seconds`;
    debug(`Finished querying ${pagedKeys.length} EVM.AccountCodes storage keys in ${keyText} ✅`);

    count = 0;
    t0 = performance.now();
    loggingFrequency = 10;
    t1 = t0;
    loopCount = 0;

    for (let i = 0; i < pagedKeys.length; i += limit) {
      const batch = pagedKeys.slice(i, i + limit);
      const returnedValues = (await limiter.schedule(() =>
        context.polkadotApi.rpc.state.queryStorageAt(batch, blockHash)
      )) as any[];

      const combined = returnedValues.map((contract, index) => ({
        contract,
        batch: batch[index],
      }));

      for (const item of combined) {
        totalContracts++;
        const accountId = "0x" + item.batch.slice(-40);
        const deployedBytecode = item.contract.toHex().slice(10);
        const codesize = getBytecodeSize(deployedBytecode);
        if (codesize > MAX_CONTRACT_SIZE_BYTES) {
          failedContractCodes.push({ accountId, codesize });
        }
      }
      count += batch.length;

      if (count % (loggingFrequency * limit) === 0) {
        const t2 = performance.now();
        const used = process.memoryUsage().heapUsed / 1024 / 1024;
        const duration = t2 - t1;
        const qps = (loggingFrequency * limit) / (duration / 1000);
        debug(
          `⏱️  Checked ${count} accounts, ${qps.toFixed(0)} accounts/sec, ${used.toFixed(
            0
          )} MB heap used, ${((count * 100) / pagedKeys.length).toFixed(1)}% complete`
        );
        loopCount++;
        t1 = t2;

        // Increase logging threshold after 5 prints
        if (loopCount % 5 === 0) {
          loggingFrequency = Math.floor(loggingFrequency ** growthFactor);
          debug(`⏫  Increased logging threshold to every ${loggingFrequency * limit} accounts`);
        }

        // Print estimated time left every 10 prints
        if (loopCount % 10 === 0) {
          const timeLeft = (pagedKeys.length - count) / qps;
          const text =
            timeLeft < 60
              ? `${timeLeft.toFixed(0)} seconds`
              : `${(timeLeft / 60).toFixed(0)} minutes`;
          debug(`⏲️  Estimated time left: ${text}`);
        }
      }
    }

    t3 = performance.now();
    const checkTime = (t3 - t0) / 1000;
    const text =
      checkTime < 60 ? `${checkTime.toFixed(1)} seconds` : `${(checkTime / 60).toFixed(1)} minutes`;
    debug(`Finished checking ${totalContracts} EVM.AccountCodes storage values in ${text} ✅`);
  });

  testIt("C100", `should not have excessively long account codes`, function () {
    expect(
      failedContractCodes.length,
      `Failed account codes (too long): ${failedContractCodes
        .map(({ accountId, codesize }) => `accountId: ${accountId} - ${chalk.red(codesize)} bytes`)
        .join(`, `)}`
    ).to.equal(0);

    debug(`Verified ${totalContracts} total account codes (at #${atBlockNumber})`);
  });
});
