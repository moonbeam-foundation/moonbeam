import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { BlockHash, PreimageStatus } from "@polkadot/types/interfaces";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import yargs from "yargs";
import { getApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { promiseConcurrent } from "../utils/functions";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    at: {
      type: "number",
      description: "block number",
    },
    account: {
      type: "string",
      description: "filter only specific nominator account",
    },
  }).argv;

const main = async () => {
  const api = await getApiFor(argv.url || argv.network);
  const filteredAccount = argv.account?.toLowerCase() || null;

  const atBlockNumber = argv.at || (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  const blockHash = await api.rpc.chain.getBlockHash(atBlockNumber);

  console.log(`Using block #${atBlockNumber} (${blockHash})`);

  console.log(`\n========= Retrieve accounts...`);

  const accountKeys = filteredAccount
    ? [await api.query.system.account.key(filteredAccount, blockHash)]
    : (await api.query.system.account.keysAt(blockHash)).map((k) => k.toString());

  console.log(`${accountKeys.length} accounts`);
  await promiseConcurrent(
    10,
    async (key, index) => {
      const id = `0x${key.toString().slice(32 + 32 + 34)}`;
      const accountInfo: any = await api.rpc.state.getStorage.raw(key, blockHash);
      const account = api.registry.createType("AccountInfo", accountInfo);
      if (account.consumers.toNumber() != 0) {
        console.log(`${id}: ${account.consumers.toNumber()}`);
      }
      if (!argv.verbose && index % 1000 == 0) {
        console.log(`Processing ${index}...`);
      }
    },
    accountKeys
  );

  await api.disconnect();
};

async function start() {
  try {
    await main();
  } catch (e) {
    console.error(e);
    process.exit(1);
  }
}

start();
