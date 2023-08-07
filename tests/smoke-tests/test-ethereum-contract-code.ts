import "@moonbeam-network/api-augment";
import { compactStripLength, hexToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { expect } from "chai";
import chalk from "chalk";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { processAllStorage } from "../util/storage";

const debug = require("debug")("smoke:ethereum-contract");

describeSmokeSuite("S600", `Ethereum contract bytecode should not be large`, (context, testIt) => {
  let atBlockNumber: number;
  let total: bigint = 0n;
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

    await processAllStorage(context.polkadotApi, keyPrefix, blockHash, (items) => {
      for (const item of items) {
        const codesize = getBytecodeSize(hexToU8a(item.value));
        if (codesize > MAX_CONTRACT_SIZE_BYTES) {
          const accountId = "0x" + item.key.slice(-40);
          failedContractCodes.push({ accountId, codesize });
        }
      }
      total += BigInt(items.length);
    });
    debug(`Finished querying ${total} EVM.AccountCodes✅`);
  });

  testIt("C100", `should not have excessively long account codes`, function () {
    expect(
      failedContractCodes.length,
      `Failed account codes (too long): ${failedContractCodes
        .map(({ accountId, codesize }) => `accountId: ${accountId} - ${chalk.red(codesize)} bytes`)
        .join(`, `)}`
    ).to.equal(0);

    debug(`Verified ${total} total account codes (at #${atBlockNumber})`);
  });
});
