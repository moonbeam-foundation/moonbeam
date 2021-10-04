// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import fs from "fs";
import BN from "bn.js";

import { getApiFor, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    at: {
      type: "number",
      description: "Block number to look into",
    },
  }).argv;

const genesisAddresses = {
  moonriver: [
    "0xab1949cea95bdc445951b823f21a30e6596364c1",
    "0xe30c476bf8b62fe3eee1a5914eb5f1c10c001454",
    "0x51c40f6186c987dcf924cb1a0ac87263fed9bd8b",
    "0x9b400d3a8a8d920d1ef4674095c354c9c3f929a8",
    "0x0f5e55ab26822263923da4f14b1300d2b9264b3f",
    "0x077e70374867349a2dc0fecb41b1c24d536b98ab",
    "0x71fba157ee58974a32a473dd3f0d5beeb73cfd5e",
    "0x76590b7071cc471e03144dd1d646a5f1299e1db7",
    "0xa60ec49cc4ac1fcd24f82e638d7481107bee0fc0",
    "0x83fd62188455f913423836b61671377602e5bd33",
    "0x770a3fe433bcf1cd0c0ab72221a35c61ad9d7f3d",
    "0xe840f52ccd6f2b0ad81ebaa42f1f6e8846e0c471",
    "0x7aff0f0b40fde8e3fb3dee9f5ef82dfcc8f53faa",
    "0x7497fccd6148e07b54be3ce4e3f79521b2a3267f",
    "0xc563973f278b3eeae0f71d367099ea74d94a3303",
    "0x48c2ccb7d1994acd1085d380969e8db0770360df",
    "0x029344fcdb05a7a79f770ed4fdd855f3d8b99383",
    "0xb8b4563e5bbd7b94e39a632a62a205d823e4a8ed",
    "0x7439adaf0e67f1d0ad514da2b9cc06320dbc7838",
    "0x9ec661dda5a39e14ff6091f1755016563ad82466",
    "0x4c3bf24c4597dea110a7d6c4d41d5fc161e139e7",
    "0xa59ac55cdb74339a4b2f6e59501efd1ebcf4f13a",
    "0xbef14b868712fd52c430f963a260725138da9204",
    "0xc2d7cc2df8da68613afb47c412984de551111a8a",
    "0x3c78feb354003798e36a15dbb704cdd3c1d06f6a",
    "0xeda72c8e02f29d90998fddd1c0a435596be66d6a",
    "0x4049fdf7a30f0d061ac9707c0d031a414028d16d",
    "0x0f5ff80225441781ebd9aa23671eac5a7f3c941c",
    "0x99aec99cf4f8adefd4da2b1fc11110450c4c736d",
    "0xe2413b3cc93655996f3bd0466602fd3817d63753",
    "0xd13ec9ff1815387b17d49ec3fadd6c799afa5ff5",
    "0x0cb93a7c2246be7a3350e9ac11846c5deb856ccd",
    "0xce5190864480457a95bbe595286cca480261935d",
    "0x05a1e7c971ade346babbe4555a6eb451be00a80e",
    "0xd690b084778370e6118b8b05a4c6480eff075e2d",
    "0x8e6115384b22235ba3ac911977ff5cb6dee85841",
    "0x0f3e7d0158bd3208122c4d26d47d911c1aa44d80",
    "0x9d07c19d69dd06be5d6ab458f07a241ceb33976d",
    "0xf83b3682a46a83aa0622d5508b6880be9b931051",
    "0x7394cf09d3f0065908bc64f7a4ef828d2cfc8a32",
    "0x7421332cd1e5e0b4cc98432ec46003ee8dc4e69f",
    "0x8809e65a96a973f96f7f44e38833da1b41708a63",
    "0xeb6d7ea9df53e011efaba99fe7137bdf922d8add",
    "0x7d5f0c9168866db358a5fbb0b9e3a26fa82b89b2",
    "0x807653fc48aa037a98cd431648dcec84ce880f97",
    "0x654e90a5fea414ecdfcb6f4d3f688c7cab913ab5",
    // "0x6e2b8c8734e9b0184e4b0193eeec2790c1bf2d2d",
    // "0x3abeda9f0f920fda379b59b042dd6625d9c86df3",
    // "0x2bcb75e8590f945596e44a94c6b9ba327745117a",
    // "0x461891503a7cc40cd5acd630907c940d2aa84bc8",
    // "0x6477c1006ab85e6d94e8e7371f23b782fe95ca6b",
    // "0x4828e3d2a1c4b0a90a2a125b9d53204efaf876a5",
    // "0x2869e58409ca3e286a89d8baec432b6bd42aa895",
    // "0x10a2f17d8150b76359e9ced567fc348c71a74b46",
    // "0xb728c13034c3b6c6447f399d25b097216a0081ea",
    "0x5a04d242669edf087ab5e6829b10d9556f4020a3",
    "0x6d6f646C70792f74727372790000000000000000",
    "0x6d6F646C43726f77646C6f610000000000000000",
  ].map((k) => k.toLowerCase()),
};

const POWER = 10n ** (18n - BigInt(2));
const DECIMAL_POWER = 10 ** 2;
const printMOVRs = (value: bigint) => {
  return (Number(value / POWER) / DECIMAL_POWER)
    .toFixed(2)
    .replace(/\B(?=(\d{3})+(?!\d))/g, ",")
    .padStart(14, " ");
};

const main = async () => {
  const api = await getApiFor(argv);

  const blockNumber = argv.at || (await api.rpc.chain.getBlock()).block.header.number.toNumber();
  const blockHash = (await api.rpc.chain.getBlockHash(blockNumber)).toString();

  // If we are querying a new round, get previous 2 rounds last block
  const totalSupply = await api.query.balances.totalIssuance();

  console.log(`#${blockNumber}: ${totalSupply.toBigInt()}`);

  let balances;
  if (!fs.existsSync(`data.json`)) {
    console.log(`Loading all data...`);
    balances = (await api.query.system.account.entriesAt(blockHash)) as any;
    fs.writeFileSync(`data.json`, JSON.stringify(balances));
  } else {
    balances = JSON.parse(fs.readFileSync(`data.json`).toString());
  }
  console.log(`balances: ${balances.length} accounts...`);

  let totalFree = 0n;
  let genesisAccounts = genesisAddresses.moonriver;
  let totalGenesis = 0n;
  let totalReserved = 0n;
  for (const [key, balance] of balances) {
    const account = `0x${key
      .toString()
      .substring(key.toString().length - 40)
      .toLowerCase()}`;
    totalFree += BigInt(balance.data.free);
    totalReserved += BigInt(balance.data.reserved);
    if (genesisAccounts.includes(account)) {
      totalGenesis += BigInt(balance.data.free) + BigInt(balance.data.reserved);
    }
  }
  console.log(
    `#${blockNumber}: ${printMOVRs(totalFree + totalReserved)} (free: ${printMOVRs(
      totalFree
    )} + reserved: ${printMOVRs(totalReserved)})`
  );
  console.log(`#${blockNumber}: ${printMOVRs(totalGenesis)} genesis`);
  console.log(
    `#${blockNumber}: ${printMOVRs(totalFree + totalReserved - totalGenesis)} circulating`
  );
  api.disconnect();
};

main();
