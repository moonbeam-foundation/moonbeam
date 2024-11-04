import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ONE_HOURS } from "@moonwall/util";
import { compactStripLength, hexToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import chalk from "chalk";
import { processRandomStoragePrefixes } from "../../helpers/storageQueries.js";

describeSuite({
  id: "S08",
  title: `Ethereum contract bytecode should not be large`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number;
    let totalContracts: bigint = 0n;
    const failedContractCodes: { accountId: string; codesize: number }[] = [];

    beforeAll(async function () {
      const paraApi = context.polkadotJs("para");
      const blockHash = process.env.BLOCK_NUMBER
        ? (await paraApi.rpc.chain.getBlockHash(parseInt(process.env.BLOCK_NUMBER))).toHex()
        : (await paraApi.rpc.chain.getFinalizedHead()).toHex();
      atBlockNumber = (await paraApi.rpc.chain.getHeader(blockHash)).number.toNumber();

      // taken from geth, e.g. search "MaxCodeSize":
      // https://github.com/etclabscore/core-geth/blob/master/params/vars/protocol_params.go
      const MAX_CONTRACT_SIZE_BYTES = 24576;
      const getBytecodeSize = (storageValue: Uint8Array) => {
        const [len, bytecode] = compactStripLength(storageValue);
        const hex = u8aToHex(bytecode);
        return (hex.length - 2) / 2;
      };

      // Max RPC response limit is 15728640 bytes (15MB), so pessimistically the pageLimit
      // needs to be lower than if every contract was above the MAX_CONTRACT_SIZE
      const keyPrefix = u8aToHex(
        u8aConcat(xxhashAsU8a("EVM", 128), xxhashAsU8a("AccountCodes", 128))
      );
      const t0 = performance.now();

      await processRandomStoragePrefixes(paraApi, keyPrefix, blockHash, (items) => {
        for (const item of items) {
          const codesize = getBytecodeSize(hexToU8a(item.value));
          if (codesize > MAX_CONTRACT_SIZE_BYTES) {
            const accountId = "0x" + item.key.slice(-40);
            failedContractCodes.push({ accountId, codesize });
          }
        }
        totalContracts += BigInt(items.length);
      });

      const t1 = performance.now();
      const checkTime = (t1 - t0) / 1000;
      const text =
        checkTime < 60
          ? `${checkTime.toFixed(1)} seconds`
          : `${(checkTime / 60).toFixed(1)} minutes`;

      log(`Finished checking ${totalContracts} EVM.AccountCodes storage values in ${text} ✅`);
    }, ONE_HOURS);

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
