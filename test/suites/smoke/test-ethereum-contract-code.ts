import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { THIRTY_MINS } from "@moonwall/util";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import chalk from "chalk";
import { rateLimiter } from "../../helpers/common.js";
const limiter = rateLimiter();

describeSuite({
  id: "S600",
  title: `Ethereum contract bytecode should not be large`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number;
    let totalContracts: bigint = 0n;
    const failedContractCodes: { accountId: string; codesize: number }[] = [];

    beforeAll(async function () {
      const blockHash = process.env.BLOCK_NUMBER
        ? (
            await context
              .polkadotJs({ apiName: "para" })
              .rpc.chain.getBlockHash(parseInt(process.env.BLOCK_NUMBER))
          ).toHex()
        : (await context.polkadotJs({ apiName: "para" }).rpc.chain.getFinalizedHead()).toHex();
      atBlockNumber = (
        await context.polkadotJs({ apiName: "para" }).rpc.chain.getHeader(blockHash)
      ).number.toNumber();

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
            context
              .polkadotJs({ apiName: "para" })
              .rpc.state.getKeysPaged(keyPrefix, limit, last_key, blockHash)
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
          log(
            `Queried ${count} keys @ ${qps.toFixed(0)} keys/sec, ${used.toFixed(0)} MB heap used`
          );

          // Increase logging threshold after 5 prints
          if (loopCount % 5 === 0) {
            loggingFrequency = Math.floor(loggingFrequency ** growthFactor);
            log(`⏫  Increased logging threshold to every ${loggingFrequency * limit} accounts`);
          }
        }
      }

      let t3 = performance.now();
      const keyQueryTime = (t3 - t0) / 1000;
      const keyText =
        keyQueryTime > 60
          ? `${(keyQueryTime / 60).toFixed(1)} minutes`
          : `${keyQueryTime.toFixed(1)} seconds`;

      const totalKeys = pagedKeys.length;
      log(`Finished querying ${totalKeys} EVM.AccountCodes storage keys in ${keyText} ✅`);

      count = 0;
      t0 = performance.now();
      loggingFrequency = 10;
      t1 = t0;
      loopCount = 0;

      while (pagedKeys.length) {
        let batch = [];
        for (let i = 0; i < limit && pagedKeys.length; i++) {
          batch.push(pagedKeys.pop());
        }
        const returnedValues = (await limiter.schedule(() =>
          context.polkadotJs({ apiName: "para" }).rpc.state.queryStorageAt(batch, blockHash)
        )) as any[];

        const combined = returnedValues.map((value, index) => ({
          value,
          address: batch[index],
        }));

        for (let j = 0; j < combined.length; j++) {
          totalContracts++;
          const accountId = "0x" + combined[j].address.slice(-40);
          const deployedBytecode = u8aToHex(combined[j].value.unwrap()).slice(6);
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
          log(
            `⏱️  Checked ${count} accounts, ${qps.toFixed(0)} accounts/sec, ${used.toFixed(
              0
            )} MB heap used, ${((count * 100) / totalKeys).toFixed(1)}% complete`
          );
          loopCount++;
          t1 = t2;

          // Increase logging threshold after 5 prints
          if (loopCount % 5 === 0) {
            loggingFrequency = Math.floor(loggingFrequency ** growthFactor);
            log(`⏫  Increased logging threshold to every ${loggingFrequency * limit} accounts`);
          }

          // Print estimated time left every 10 prints
          if (loopCount % 10 === 0) {
            const timeLeft = (pagedKeys.length - count) / qps;
            const text =
              timeLeft < 60
                ? `${timeLeft.toFixed(0)} seconds`
                : `${(timeLeft / 60).toFixed(0)} minutes`;
            log(`⏲️  Estimated time left: ${text}`);
          }
        }
      }

      t3 = performance.now();
      const checkTime = (t3 - t0) / 1000;
      const text =
        checkTime < 60
          ? `${checkTime.toFixed(1)} seconds`
          : `${(checkTime / 60).toFixed(1)} minutes`;
      log(`Finished checking ${totalContracts} EVM.AccountCodes storage values in ${text} ✅`);
    }, THIRTY_MINS);

    it({
      id: "C100",
      title: "should not have excessively long account codes",
      test: async function () {
        expect(
          failedContractCodes.length,
          `Failed account codes (too long): ${failedContractCodes
            .map(
              ({ accountId, codesize }) => `accountId: ${accountId} - ${chalk.red(codesize)} bytes`
            )
            .join(`, `)}`
        ).to.equal(0);

        log(`Verified ${totalContracts} total account codes (at #${atBlockNumber})`);
      },
    });
  },
});
