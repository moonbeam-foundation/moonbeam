import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { BlockHash, PreimageStatus } from "@polkadot/types/interfaces";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import yargs from "yargs";
import { NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { promiseConcurrent } from "../utils/functions";

// TODO: clean the code, use PolkadotJS for storage key and ...
// TODO: use multi or range to query multiple accounts at once
// TODO: add democracy propose/second support
// TODO: add cost for proxy > 2
// WARNING: This is not accurate (missing democracy propose/second and maybe others...)

const debug = require("debug")("main");

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
    decimals: {
      type: "number",
      default: 0,
      description: "number of decimal under MOVR",
    },
  }).argv;

const POWER = 10n ** (18n - BigInt(argv.decimals));
const DECIMAL_POWER = 10 ** argv.decimals;
const printMOVRs = (value: bigint) => {
  if (argv.decimals > 0) {
    return (Number(value / POWER) / DECIMAL_POWER)
      .toFixed(argv.decimals)
      .padStart(6 + argv.decimals, " ");
  }
  return (value / POWER).toString().padStart(6, " ");
};

const getStorageKey = (module: string, name: string, account?: string) => {
  if (account) {
    return u8aToHex(
      u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128), xxhashAsU8a(account, 64), account)
    );
  }
  return u8aToHex(u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128)));
};

const getStorageMap = async (
  polkadotApi: ApiPromise,
  module: string,
  name: string,
  type: string,
  at: BlockHash
) => {
  const keys: any = await polkadotApi.rpc.state.getKeys(getStorageKey(module, name), at);
  return (
    await Promise.all(
      keys.map(async (key) => {
        const data: any = await polkadotApi.rpc.state.getStorage.raw(key, at);
        return {
          key,
          data: polkadotApi.registry.createType(type, data),
        };
      })
    )
  ).reduce((p, { key, data }) => {
    p[key] = data;
    return p;
  }, {});
};

const getTwox64AccountStorageMap = async (
  polkadotApi: ApiPromise,
  module: string,
  name: string,
  type: string,
  at: BlockHash,
  account?: string
) => {
  const keys: any = account
    ? [getStorageKey(module, name, account)]
    : await polkadotApi.rpc.state.getKeys(getStorageKey(module, name), at);

  return (
    await Promise.all(
      keys.map(async (key) => {
        const id = `0x${key.toString().slice(32 + 32 + 18)}`;
        if (account && account != id) {
          return null;
        }
        const data: any = await polkadotApi.rpc.state.getStorage.raw(key, at);
        if (data.length == 0) {
          debug(`${id.substring(0, 7)}...${id.substring(id.length - 4)}: not found`);
          return null;
        }
        return {
          id,
          data: polkadotApi.registry.createType(type, data),
        };
      })
    )
  )
    .filter((v) => !!v)
    .reduce((p, { id, data }) => {
      p[id] = data;
      return p;
    }, {});
};

const getNominatorsStakes = async (polkadotApi: ApiPromise, at: BlockHash, account?: string) => {
  const nominators = (await getTwox64AccountStorageMap(
    polkadotApi,
    "ParachainStaking",
    "NominatorState2",
    "Nominator2",
    at,
    account
  )) as { [accountId: string]: any };

  Object.keys(nominators)
    .sort()
    .map((accountId) => {
      debug(
        `${accountId.substring(0, 7)}...${accountId.substring(accountId.length - 4)}: ${
          nominators[accountId].total.toBigInt() / 1000000000000000000n
        } (${nominators[accountId].nominations
          .map(
            (m) =>
              `${m.owner.toString().substring(0, 7)}...${m.owner
                .toString()
                .substring(m.owner.toString().length - 4)}: ${
                BigInt(m.amount) / 1000000000000000000n
              }`
          )
          .join(", ")})`
      );
    });

  const stakes: { [accountId: string]: bigint } = {};
  Object.keys(nominators).map((key) => {
    stakes[key] = nominators[key].total.toBigInt();
  });
  return stakes;
};

const collatorToString = (accountId: string, collator: any) => {
  return `${accountId}:  ${collator.nominators.length
    .toString()
    .padEnd(3, " ")} nominations [bond: ${(collator.bond.toBigInt() / 10n ** 18n)
    .toString()
    .padEnd(5, " ")}, backing: ${(collator.total_backing.toBigInt() / 10n ** 18n)
    .toString()
    .padEnd(5, " ")}, counted: ${(collator.total_counted.toBigInt() / 10n ** 18n)
    .toString()
    .padEnd(5, " ")}] (top: ${collator.top_nominators.length.toString().padEnd(3, " ")}[amount: ${(
    collator.top_nominators.reduce((p, nom) => p + nom.amount.toBigInt(), 0n) /
    10n ** 18n
  )
    .toString()
    .padEnd(5, " ")}], bottom: ${collator.bottom_nominators.length
    .toString()
    .padStart(3, " ")} [amount: ${(
    collator.bottom_nominators.reduce((p, nom) => p + nom.amount.toBigInt(), 0n) /
    10n ** 18n
  )
    .toString()
    .padEnd(5, " ")}])`;
};

const getCollatorsStates = async (polkadotApi: ApiPromise, at: BlockHash, account?: string) => {
  const collators = (await getTwox64AccountStorageMap(
    polkadotApi,
    "ParachainStaking",
    "CollatorState2",
    "Collator2",
    at,
    account
  )) as { [accountId: string]: any };

  Object.keys(collators)
    .sort()
    .map((accountId) => {
      debug(collatorToString(accountId, collators[accountId]));
    });

  return collators;
};

const main = async () => {
  const wsProvider = new WsProvider(argv.url || argv.network);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });
  const filteredAccount = argv.account?.toLowerCase() || null;

  const atBlockNumber =
    argv.at || (await polkadotApi.rpc.chain.getBlock()).block.header.number.toNumber();
  const blockHash = await polkadotApi.rpc.chain.getBlockHash(atBlockNumber);

  console.log(`Using block #${atBlockNumber} (${blockHash})`);

  console.log(`\n========= Retrieve collators/nominators/proxy storage...`);

  const nominatorStakes = await getNominatorsStakes(polkadotApi, blockHash, filteredAccount);
  const collators = await getCollatorsStates(polkadotApi, blockHash, filteredAccount);
  const proxies = (await getTwox64AccountStorageMap(
    polkadotApi,
    "Proxy",
    "Proxies",
    "Vec<ProxyDefinition>",
    blockHash,
    filteredAccount
  )) as { [accountId: string]: any };
  const mappings = (await getTwox64AccountStorageMap(
    polkadotApi,
    "AuthorMapping",
    "MappingWithDeposit",
    "RegistrationInfo",
    blockHash
  )) as { [accountId: string]: any };

  const authorMappings = Object.keys(mappings)
    .filter((key) => {
      return !filteredAccount || mappings[key].account.toString().toLowerCase() == filteredAccount;
    })
    .reduce((p, key) => {
      p[mappings[key].account.toString().toLowerCase()] = mappings;
      return p;
    }, {});

  const preimages = (await getStorageMap(
    polkadotApi,
    "Democracy",
    "Preimages",
    "PreimageStatus",
    blockHash
  )) as { [key: string]: PreimageStatus };
  const accountPreimages = Object.values(preimages)
    .filter((preimage) => preimage.isAvailable)
    .reduce((p, preimage) => {
      const account = preimage.asAvailable.provider.toString().toLowerCase();
      p[account] = (p[account] || 0n) + preimage.asAvailable.deposit.toBigInt();
      return p;
    }, {});

  const accountKeys = filteredAccount
    ? [await polkadotApi.query.system.account.key(filteredAccount, blockHash)]
    : (await polkadotApi.query.system.account.keysAt(blockHash)).map((k) => k.toString());

  console.log(`${accountKeys.length} accounts`);
  await promiseConcurrent(
    10,
    async (key, index) => {
      const id = `0x${key.toString().slice(32 + 32 + 34)}`;
      const accountInfo: any = await polkadotApi.rpc.state.getStorage.raw(key, blockHash);
      const account = polkadotApi.registry.createType("AccountInfo", accountInfo);
      const proxyCount = proxies[id]?.length || 0;
      const proxyCost =
        proxyCount == 0
          ? 0n
          : proxyCount == 1
          ? 1002900000000000000n
          : proxyCount == 2
          ? 1005000000000000000n
          : BigInt(proxyCount);

      const authorMappingCost = authorMappings[id] ? 100n * 10n ** 18n : 0n;
      const preimageCost = accountPreimages[id] || 0n;
      if (
        (argv.verbose && account.data.reserved.toBigInt() != 0n) ||
        account.data.reserved.toBigInt() !=
          (nominatorStakes[id] || 0n) +
            (collators[id]?.bond?.toBigInt() || 0n) +
            proxyCost +
            authorMappingCost +
            preimageCost
      ) {
        console.log(
          `${id} [reserved: ${printMOVRs(
            account.data.reserved.toBigInt()
          )}][nominated: ${printMOVRs(nominatorStakes[id] || 0n)}][bonded: ${printMOVRs(
            collators[id]?.bond?.toBigInt() || 0n
          )}][proxy: ${printMOVRs(proxyCost)}][mapping: ${printMOVRs(
            authorMappingCost
          )}][preimage: ${printMOVRs(preimageCost)}]`
        );
      }
      if (!argv.verbose && index % 1000 == 0) {
        console.log(`Processing ${index}...`);
      }
    },
    accountKeys
  );

  await polkadotApi.disconnect();
};

main();
