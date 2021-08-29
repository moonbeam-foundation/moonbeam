import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { BlockHash } from "@polkadot/types/interfaces";
import { exploreBlockRange } from "../utils/monitoring";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import yargs from "yargs";
import { NETWORK_YARGS_OPTIONS } from "../utils/networks";
const debug = require("debug")("main");

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    from: {
      type: "number",
      description: "from block number (included)",
      demandOption: true,
    },
    to: {
      type: "number",
      description: "to block number (included)",
      demandOption: true,
    },
    account: {
      type: "string",
      description: "filter only specific nominator account",
    },
    "scan-storage": {
      type: "boolean",
      description: "scan storage for difference between each block",
      implies: ["account"],
    },
  }).argv;

const nominatorState2Key = u8aToHex(
  u8aConcat(xxhashAsU8a("ParachainStaking", 128), xxhashAsU8a("NominatorState2", 128))
);
const nominatorState2AccountKey = (account: string) =>
  u8aToHex(
    u8aConcat(
      xxhashAsU8a("ParachainStaking", 128),
      xxhashAsU8a("NominatorState2", 128),
      xxhashAsU8a(account, 64),
      account
    )
  );

const getNominatorsData = async (polkadotApi: ApiPromise, at: BlockHash, account?: string) => {
  const nominatorKeys: any = account
    ? [nominatorState2AccountKey(account)]
    : await polkadotApi.rpc.state.getKeys(nominatorState2Key, at);
  const bonded = {};
  for (const key of nominatorKeys) {
    const id = `0x${key.toString().slice(32 + 32 + 18)}`;
    if (account && account != id) {
      continue;
    }
    const data: any = await polkadotApi.rpc.state.getStorage.raw(key, at);
    if (data.length == 0) {
      debug(`${id.substring(0, 7)}: not found`);
      continue;
    }
    const nominator = polkadotApi.registry.createType("Nominator2", data) as any;
    bonded[id] = BigInt(nominator.total);

    debug(
      `${id.substring(0, 7)}: ${bonded[id] / 1000000000000000000n} (${nominator.nominations
        .map(
          (m) => `${m.owner.toString().substring(0, 7)}: ${BigInt(m.amount) / 1000000000000000000n}`
        )
        .join(", ")})`
    );
  }
  return bonded;
};

const main = async () => {
  const wsProvider = new WsProvider("ws://localhost:56992");
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });
  const filteredAccount = argv.account?.toLowerCase() || null;

  const fromHash = await polkadotApi.rpc.chain.getBlockHash(argv.from - 1);
  const toHash = await polkadotApi.rpc.chain.getBlockHash(argv.to);

  const bonded = await getNominatorsData(polkadotApi, fromHash, filteredAccount);

  if (filteredAccount) {
    console.log(
      `#${argv.from - 1}: Storage (${filteredAccount}: ${
        bonded[filteredAccount] ? bonded[filteredAccount] / 1000000000000000000n : `not found`
      })`
    );
  }

  await exploreBlockRange(
    polkadotApi,
    { from: argv.from, to: argv.to, concurrency: 5 },
    async (blockDetails) => {
      blockDetails.records.forEach(({ event }, index) => {
        const types = event.typeDef;
        if (event.section == "parachainStaking" && event.method == "JoinedCollatorCandidates") {
          // Doesn't add to the nominator list
          const [acc, bond, newTotal] = event.data;
          const key = acc.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }
          // bonded[key] = (bonded[key] || 0n) + (bond as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: Skipped - ${event.method} (${key} + ${(
              bond as any
            ).toBigInt()})`
          );
        }
        // collators are not in nominators list
        if (event.section == "parachainStaking" && event.method == "CollatorBondedMore") {
          const [collator, before, after] = event.data;
          const key = collator.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }
          // bonded[key] =
          //   (bonded[key] || 0n) + (after as any).toBigInt() - (before as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: Skipped - ${event.method} (${key} + ${
              (after as any).toBigInt() - (before as any).toBigInt()
            })`
          );
        }
        if (event.section == "parachainStaking" && event.method == "Nomination") {
          const [acc, amount, collator, nominator_position] = event.data;
          const key = acc.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }
          bonded[key] = (bonded[key] || 0n) + (amount as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: ${event.method} (${key} + ${(
              amount as any
            ).toBigInt()})`
          );
        } else if (event.section == "parachainStaking" && event.method == "NominationIncreased") {
          const [nominator, candidate, before, in_top, after] = event.data;
          const key = nominator.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }

          const previousEvent = blockDetails.records[index - 1].event;
          if (previousEvent.section != "balances" && previousEvent.method != "Reserved") {
            console.log(
              `Unexpected event before NominationIncreased at #${blockDetails.block.header.number}`
            );
            process.exit(1);
          }
          const [account, amount] = previousEvent.data;
          if (account.toString().toLowerCase() != key) {
            console.log(
              `Unexpected account for event balances.Reserved before NominationIncreased at #${blockDetails.block.header.number}`
            );
            process.exit(1);
          }
          bonded[key] = (bonded[key] || 0n) + (amount as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: ${event.method} (${key} + ${(
              amount as any
            ).toBigInt()})`
          );
        } else if (event.section == "parachainStaking" && event.method == "CollatorBondedLess") {
          // collators are not in nominators list
          const [collator, before, after] = event.data;
          const key = collator.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }
          // bonded[key] =
          //   (bonded[key] || 0n) + (after as any).toBigInt() - (before as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: Skipped - ${event.method} (${key} + ${
              (after as any).toBigInt() - (before as any).toBigInt()
            })`
          );
        } else if (event.section == "parachainStaking" && event.method == "NominationDecreased") {
          const [nominator, candidate, before, in_top, after] = event.data;
          const key = nominator.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }
          const previousEvent = blockDetails.records[index - 1].event;
          if (previousEvent.section != "balances" && previousEvent.method != "Unreserved") {
            console.log(
              `Unexpected event before NominationIncreased at #${blockDetails.block.header.number}`
            );
            process.exit(1);
          }
          const [account, amount] = previousEvent.data;
          if (account.toString().toLowerCase() != key) {
            console.log(
              `Unexpected account for event balances.Unreserved before NominationIncreased at #${blockDetails.block.header.number}`
            );
            process.exit(1);
          }
          bonded[key] = (bonded[key] || 0n) - (amount as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: ${event.method} (${key} - ${(
              amount as any
            ).toBigInt()})`
          );
        } else if (event.section == "parachainStaking" && event.method == "NominatorLeftCollator") {
          const [nominator, collator, nominator_stake, new_total] = event.data;
          const key = nominator.toString().toLowerCase();
          if (filteredAccount && filteredAccount != key) {
            return;
          }
          bonded[key] = (bonded[key] || 0n) - (nominator_stake as any).toBigInt();
          console.log(
            `#${blockDetails.block.header.number}: ${event.method} (${key} - ${(
              nominator_stake as any
            ).toBigInt()})`
          );
        } else if (event.section == "parachainStaking" && event.method == "CollatorLeft") {
          const [nominator, total_backing, new_total_staked] = event.data;
          const key = nominator.toString().toLowerCase();

          // This can trigger multiple unreserved

          if (filteredAccount && filteredAccount != key) {
            return;
          }
          for (let i = index - 1; i > 0; i--) {
            const previousEvent = blockDetails.records[i].event;
            if (previousEvent.section != "balances" && previousEvent.method != "Unreserved") {
              break;
            }
            const [account, amount] = previousEvent.data;
            const accountKey = account.toString().toLowerCase();
            if (filteredAccount && filteredAccount != accountKey) {
              continue;
            }
            console.log(
              `#${blockDetails.block.header.number}: ${accountKey == key ? "Skipped - " : ""}${
                event.method
              }-${i} (${accountKey} - ${(amount as any).toBigInt()})`
            );
            if (accountKey != key) {
              bonded[accountKey] = (bonded[accountKey] || 0n) - (amount as any).toBigInt();
            }
          }
        } else {
          return;
        }

        // console.log(`Block: ${blockDetails.block.header.number}`);
        // console.log(`\t${event.section}:${event.method}`);
        // console.log(`\t\t${event.meta.toString()}`);

        // // Loop through each of the parameters, displaying the type and data
        // event.data.forEach((data, index) => {
        //   console.log(`\t\t\t${types[index].type}: ${data.toString()}`);
        // });
      });
      if (filteredAccount && argv["scan-storage"]) {
        const bond = await getNominatorsData(
          polkadotApi,
          blockDetails.block.header.hash,
          filteredAccount
        );
        if (bond[filteredAccount] != bonded[filteredAccount]) {
          console.log(`Block: ${blockDetails.block.header.number}`);
          console.log(
            `${filteredAccount}: (expected: ${bonded[filteredAccount]}, storage: ${bond[filteredAccount]})`
          );
          process.exit(1);
        }
      }
    }
  );

  const endBonded = await getNominatorsData(polkadotApi, toHash, filteredAccount);

  if (filteredAccount) {
    console.log(
      `#${argv.to}: Storage (${filteredAccount}: ${
        endBonded[filteredAccount] ? endBonded[filteredAccount] / 1000000000000000000n : `not found`
      })`
    );
  }

  console.log(`\n=========Checking`);
  for (const id of Object.keys(endBonded)) {
    if (bonded[id] != endBonded[id]) {
      const balance = (await polkadotApi.query.system.account.at(toHash, id)).data;
      console.log(
        `${id}: (storage: ${endBonded[id]}, computed: ${
          bonded[id]
        }), free: ${balance.free.toString()}, reversed: ${balance.reserved.toString()}`
      );
    }
  }
  console.log(``);
  for (const id of Object.keys(bonded)) {
    if (bonded[id] && !endBonded[id]) {
      const balance = (await polkadotApi.query.system.account.at(toHash, id)).data;
      console.log(
        `${id}: (storage: ${endBonded[id]}, computed: ${
          bonded[id]
        }), free: ${balance.free.toString()}, reversed: ${balance.reserved.toString()}`
      );
    }
  }

  await polkadotApi.disconnect();
};

main();
