import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { BlockHash, Extrinsic } from "@polkadot/types/interfaces";
import { exploreBlockRange } from "../utils/monitoring";
import { xxhashAsU8a, blake2AsHex } from "@polkadot/util-crypto";
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
    },
    account: {
      type: "string",
      description: "filter only specific nominator account",
    },
    "council-private-key": {
      type: "string",
      description: "Private key for council to send proposal",
    },
    "send-external-proposal": {
      type: "boolean",
      description: "should send external proposal",
      implies: ["council-private-key", "collective-threshold"],
    },
    "collective-threshold": {
      type: "number",
      demandOption: false,
    },
    "scan-storage": {
      type: "boolean",
      description: "scan storage for difference between each block",
    },
  }).argv;

const parachainStorageKey = (name: string, account?: string) => {
  if (account) {
    return u8aToHex(
      u8aConcat(
        xxhashAsU8a("ParachainStaking", 128),
        xxhashAsU8a(name, 128),
        xxhashAsU8a(account, 64),
        account
      )
    );
  }
  return u8aToHex(u8aConcat(xxhashAsU8a("ParachainStaking", 128), xxhashAsU8a(name, 128)));
};

const getParachainStorageData = async (
  polkadotApi: ApiPromise,
  name: string,
  type: string,
  at: BlockHash,
  account?: string
) => {
  const nominatorKeys: any = account
    ? [parachainStorageKey(name, account)]
    : await polkadotApi.rpc.state.getKeys(parachainStorageKey(name), at);

  return (
    await Promise.all(
      nominatorKeys.map(async (key) => {
        const id = `0x${key.toString().slice(32 + 32 + 18)}`;
        if (account && account != id) {
          return null;
        }
        const data: any = await polkadotApi.rpc.state.getStorage.raw(key, at);
        if (data.length == 0) {
          debug(`${id.substring(0, 7)}: not found`);
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
  const nominators = (await getParachainStorageData(
    polkadotApi,
    "NominatorState2",
    "Nominator2",
    at,
    account
  )) as { [accountId: string]: any };

  Object.keys(nominators)
    .sort()
    .map((accountId) => {
      debug(
        `${accountId.substring(0, 7)}: ${
          nominators[accountId].total.toBigInt() / 1000000000000000000n
        } (${nominators[accountId].nominations
          .map(
            (m) =>
              `${m.owner.toString().substring(0, 7)}: ${BigInt(m.amount) / 1000000000000000000n}`
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
  return `${accountId.substring(0, 7)}:  ${collator.nominators.length} nominations (top: ${
    collator.top_nominators.length
  }, bottom: ${collator.bottom_nominators.length})`;
};

const getCollatorsStates = async (polkadotApi: ApiPromise, at: BlockHash, account?: string) => {
  const collators = (await getParachainStorageData(
    polkadotApi,
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
  const wsProvider = new WsProvider("ws://localhost:56992");
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });
  const filteredAccount = argv.account?.toLowerCase() || null;

  const toBlockNumber =
    argv.to || (await polkadotApi.rpc.chain.getBlock()).block.header.number.toNumber();
  const fromBlockNumber = argv.from;
  console.log(
    `Using range #${fromBlockNumber}-#${toBlockNumber} (${toBlockNumber - fromBlockNumber + 1})`
  );

  const previousHash = await polkadotApi.rpc.chain.getBlockHash(fromBlockNumber - 1);
  const toHash = await polkadotApi.rpc.chain.getBlockHash(toBlockNumber);

  const nominatorBonds = await getNominatorsStakes(polkadotApi, previousHash, filteredAccount);
  const collators = await getCollatorsStates(polkadotApi, previousHash, filteredAccount);

  if (filteredAccount) {
    console.log(
      `#${fromBlockNumber - 1}: Nominator (${filteredAccount}: ${
        nominatorBonds[filteredAccount]
          ? nominatorBonds[filteredAccount] / 1000000000000000000n
          : `not found`
      })`
    );
    console.log(
      `#${fromBlockNumber - 1}: Collator - ${
        collators[filteredAccount]
          ? `${collatorToString(filteredAccount, collators[filteredAccount])})`
          : `(${filteredAccount}: not found)`
      }`
    );
  }

  // Stores collator once their storage mismatch
  const reportedCollators = {};

  await exploreBlockRange(
    polkadotApi,
    { from: fromBlockNumber, to: toBlockNumber, concurrency: 5 },
    async (blockDetails) => {
      if (argv["scan-storage"]) {
        const collators = await getCollatorsStates(
          polkadotApi,
          blockDetails.block.hash,
          filteredAccount
        );
        for (const accountId of Object.keys(collators)) {
          const collator = collators[accountId];

          if (
            !reportedCollators[accountId] && // Do not repeat the error at each block
            collator.nominators.length !=
              collator.top_nominators.length + collator.bottom_nominators.length
          ) {
            console.log(
              `#${blockDetails.block.header.number}: ERROR - ${collatorToString(
                accountId,
                collator
              )}`
            );
            reportedCollators[accountId] = `Invalid #${
              blockDetails.block.header.number
            } ${collatorToString(accountId, collator)} `;
          }
        }
      }

      // console.log(`block: #${blockDetails.block.header.number}`);
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
          nominatorBonds[key] = (nominatorBonds[key] || 0n) + (amount as any).toBigInt();
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
          nominatorBonds[key] = (nominatorBonds[key] || 0n) + (amount as any).toBigInt();
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
          nominatorBonds[key] = (nominatorBonds[key] || 0n) - (amount as any).toBigInt();
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
          nominatorBonds[key] = (nominatorBonds[key] || 0n) - (nominator_stake as any).toBigInt();
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
              nominatorBonds[accountKey] =
                (nominatorBonds[accountKey] || 0n) - (amount as any).toBigInt();
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
        const bond = await getNominatorsStakes(
          polkadotApi,
          blockDetails.block.header.hash,
          filteredAccount
        );
        if (bond[filteredAccount] != nominatorBonds[filteredAccount]) {
          console.log(`Block: ${blockDetails.block.header.number}`);
          console.log(
            `${filteredAccount}: (expected: ${nominatorBonds[filteredAccount]}, storage: ${bond[filteredAccount]})`
          );
          process.exit(1);
        }
      }
    }
  );

  const endBonded = await getNominatorsStakes(polkadotApi, toHash, filteredAccount);

  if (filteredAccount) {
    console.log(
      `#${toBlockNumber}: Storage (${filteredAccount}: ${
        endBonded[filteredAccount] ? endBonded[filteredAccount] / 1000000000000000000n : `not found`
      })`
    );
  }

  const accountsToFix: { id: string; stored: bigint; computed: bigint; reserved: bigint }[] = [];

  console.log(`\n=========Checking`);
  for (const id of [
    ...Object.keys(endBonded),
    ...Object.keys(nominatorBonds).filter((b) => !endBonded[b]),
  ]) {
    // convert storage undefined to 0
    if (nominatorBonds[id] != (endBonded[id] || 0n)) {
      const balance = ((await polkadotApi.query.system.account.at(toHash, id)) as any).data;
      console.log(
        `${id}: (storage: ${endBonded[id]}, computed: ${
          nominatorBonds[id]
        }), free: ${balance.free.toString()}, reversed: ${balance.reserved.toString()}`
      );

      accountsToFix.push({
        id,
        stored: endBonded[id] || 0n,
        computed: nominatorBonds[id],
        reserved: balance.reserved.toBigInt(),
      });
    }
  }
  console.log(``);

  // Reported collators having bad nomiations vs top_nomination+bottom_nominations
  for (const accountId of Object.keys(reportedCollators)) {
    console.log(reportedCollators[accountId]);
  }

  if (argv["send-external-proposal"]) {
    console.log(`\nPreparing external proposal`);

    const keyring = new Keyring({ type: "ethereum" });
    const council = await keyring.addFromUri(argv["council-private-key"], null, "ethereum");
    const { nonce: rawNonce } = await polkadotApi.query.system.account(council.address);
    let nonce = BigInt(rawNonce.toString());

    const unreserveTxs: Extrinsic[] = [];

    for (const account of accountsToFix) {
      if (account.reserved != account.computed) {
        console.log(
          `SKIPPED ${
            account.id
          }, reserved (${account.reserved.toString()}) != computed (${account.computed.toString()})`
        );
        continue;
      }
      console.log(`Unreserving ${account.reserved - account.stored} for ${account.id}`);
      unreserveTxs.push(
        await polkadotApi.tx.parachainStaking.unreserve_hotfix(
          account.id,
          account.reserved - account.stored
        )
      );
    }
    const batchTx = polkadotApi.tx.utility.batchAll(unreserveTxs);
    const encodedProposal = batchTx?.method.toHex() || "";
    const encodedHash = blake2AsHex(encodedProposal);

    console.log(`Encoded proposal hash ${encodedHash}`);
    console.log(`Encoded length ${encodedProposal.length}`);

    console.log(`Sending Preimage Note...`);
    // await polkadotApi.tx.democracy
    //   .notePreimage(encodedProposal)
    //   .signAndSend(council, { nonce: nonce++ });

    const external = polkadotApi.tx.democracy.externalProposeMajority(encodedHash);

    console.log(`Sending Council ...`);
    // await polkadotApi.tx.councilCollective
    //   .propose(argv["collective-threshold"], external, external.length)
    //   .signAndSend(council, { nonce: nonce++ });
  }

  await polkadotApi.disconnect();
};

main();
