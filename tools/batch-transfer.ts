// Script to do a batch of transfers using a CSV file containing format:

// 0x4BD99C921C410B4129C129B9Ae8edEF2522C4c4c,505
// 0xCFA4879A0Ba0D59b097C3b84401c51f120b98665,255
// 0x217fDd8B6cDF8F3f8f2388C7aF749A6F9cc1034A,55
// 0x22149c295D3CC843E2A14ABe93D33e0765Fb18c4,55

import { Keyring } from "@polkadot/api";
import chalk from "chalk";
import fs from "fs";
import yargs from "yargs";

import { getApiFor, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    csv: {
      type: "string",
      description: "csv file to read from (format: 0x1234.....,150)",
      demandOption: true,
    },
    "account-priv-key": {
      type: "string",
      description: "Key of the account to transfer from",
    },
    execute: {
      type: "boolean",
      description: "execute the transaction (safety flag)",
      implies: ["account-priv-key"],
    },
  }).argv;

const main = async () => {
  const api = await getApiFor(argv.url || argv.network);
  const csvData = fs.readFileSync(argv.csv);
  const lines = csvData.toString().split(/\r?\n/);
  const transfers = lines.map((l, index) => {
    const data = l.split(",");
    if (data.length != 2) {
      throw new Error(`Invalid data line ${index + 1}`);
    }
    const account = data[0];
    const amount = Number(data[1]);

    if (!/^(0x){1}[0-9a-fA-F]{40}$/i.test(account)) {
      throw new Error(`Invalid data line ${index + 1}: ${account} is not an Ethereum address`);
    }
    // Randomly decided 10000 to be the limit, to avoid people entering 18 decimals numbers
    if (isNaN(amount) || amount <= 0 || amount > 10000) {
      throw new Error(`Invalid data line ${index + 1}: amount ${amount} is not valid`);
    }
    return {
      account: data[0],
      amount: Number(data[1]),
    };
  });
  const total = transfers.reduce((p, { account, amount }) => p + amount, 0);
  console.log(`Found ${lines.length} transfers for a total of ${total} Tokens`);

  if (lines.length > 1000) {
    throw new Error(`Too many transfers. Limited to 1000`);
  }

  if (!argv.execute) {
    console.log(
      `${chalk.red("Skipping")} the transaction (add --execute to execute the transaction)`
    );
  }

  if (argv.execute) {
    const keyring = new Keyring({ type: "ethereum" });
    const account = await keyring.addFromUri(argv["account-priv-key"]);

    const balance = (await api.query.system.account(account.address)).data.free;
    if (balance.toBigInt() < BigInt(total + 1) * 10n ** 18n) {
      // 10 for fees
      throw new Error(`Balance for ${account.address} is too low: ${balance.toHuman()}`);
    }

    console.log(`Account ${account.address} has ${chalk.green(balance.toHuman())}`);

    console.log(`${chalk.red("Executing")} the transaction...`);
    const txs = transfers.map(({ account, amount }) =>
      api.tx.balances.transfer(account, BigInt(amount) * 10n ** 18n)
    );

    // Putting all the transactions in a batchAll
    // await api.tx.utility.batchAll(txs).send();
    await new Promise(async (resolve) => {
      const unsub = await api.tx.utility
        .batchAll(txs)
        .signAndSend(account, {}, ({ events = [], status }) => {
          console.log(
            `Transaction status: ${
              status.type == "Ready" ? chalk.yellow(status.type) : chalk.green(status.type)
            }`
          );

          if (status.isInBlock) {
            console.log(`Included at block hash ${chalk.green(status.asInBlock.toHex())}`);
          } else if (status.isFinalized) {
            console.log(`Finalized block hash ${status.asFinalized.toHex()}`);
            unsub();
            resolve(null);
          }
        });
    });
  }
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
