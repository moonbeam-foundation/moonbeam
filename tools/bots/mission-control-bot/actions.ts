import { MessageEmbed, Message } from "discord.js";
import Web3 from "web3";

import { UserToTimestamp, BalanceCheck } from "./types";
import { TOKEN_DECIMAL, EMBED_COLOR_CORRECT, EMBED_COLOR_ERROR, params } from "./constants";
import {
  sendSlackNotification,
  nextAvailableToken,
  checkH160AddressIsCorrect,
  sleep,
} from "./utils";

/**
 * Waits for the request to be on top of the pending queue
 * @param authorId The user ID requesting the funds on Discord
 * @param address Address that the user requested funds to
 * @returns
 */
const waitForQueue = async (authorId: string, address: string, pendingQueue: string[]) => {
  if (pendingQueue.length === 0) return;

  while (true) {
    if (pendingQueue[0] == `${authorId}:0x${address}`) break;

    // wait for next block
    await sleep(6.2);
  }
};

/**
 * Action for the bot for the pattern "!faucet send <h160_addr>", that
 * sends funds to the indicated account.
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 * @param msg Received discord message object
 * @param authorId Author ID of the message
 * @param messageContent Content of the message
 * @param receivers Map with the timestamp of the last received request of a user
 * @param lastBalanceCheck Object with the info of the last balance check of the account of the bot
 * @param pendingQueue Queue of tasks
 */
export async function botActionFaucetSend(
  web3Api: Web3,
  msg: Message,
  authorId: string,
  messageContent: string,
  receivers: UserToTimestamp,
  lastBalanceCheck: BalanceCheck,
  pendingQueue: string[]
) {
  // set default value of lastReceived at 0
  if (!receivers[authorId]) receivers[authorId] = 0;

  const canReceiveTokensAgain =
    params.NOT_LIMITED_USERS.includes(authorId) ||
    receivers[authorId] <= Date.now() - params.FAUCET_SEND_INTERVAL * 3600 * 1000;

  if (!canReceiveTokensAgain) {
    const errorEmbed = new MessageEmbed()
      .setColor(EMBED_COLOR_ERROR)
      .setTitle(`You already received tokens!`)
      .addField(
        "Remaining time",
        `You still need to wait ${nextAvailableToken(
          params.FAUCET_SEND_INTERVAL,
          receivers[authorId]
        )} to receive more tokens`
      )
      .setFooter(
        `Funds transactions are limited to once every ${params.FAUCET_SEND_INTERVAL} hour(s)`
      );

    msg.channel.send(errorEmbed);
    return;
  }

  let address = messageContent.slice("!faucet send".length).trim();
  if (address.startsWith("0x")) {
    address = address.slice(2);
  }

  // check address and send alert msg and return if bad formatted
  if (!checkH160AddressIsCorrect(address, msg)) return;

  // update user's last fund retrieval
  const previousRequestTime = receivers[authorId];
  receivers[authorId] = Date.now();

  try {
    // push to TODO queue
    pendingQueue.push(`${authorId}:0x${address}`);

    // wait for our item to be first in the list
    await waitForQueue(authorId, address, pendingQueue);

    // send tx to the chain
    await web3Api.eth.sendSignedTransaction(
      (
        await web3Api.eth.accounts.signTransaction(
          {
            value: `${params.TOKEN_COUNT * 10n ** TOKEN_DECIMAL}`,
            gasPrice: "0",
            gas: "21000",
            to: `0x${address}`,
          },
          params.ACCOUNT_KEY
        )
      ).rawTransaction
    );

    // once our tx is processed, remove it from queue
    pendingQueue.shift();
  } catch (error) {
    // rollback the update of user's last fund retrieval
    receivers[authorId] = previousRequestTime;

    // remove failed tx from queue
    pendingQueue.shift();

    // alert in channel
    const errorEmbed = new MessageEmbed()
      .setColor(EMBED_COLOR_ERROR)
      .setTitle("Could not submit the transaction")
      .setFooter(
        "The transaction of funds could not be submitted. " + "Please, try requesting funds again."
      );

    // send message
    msg.channel.send(errorEmbed);

    throw error;
  }

  const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));
  const fundsTransactionEmbed = new MessageEmbed()
    .setColor(EMBED_COLOR_CORRECT)
    .setTitle("Transaction of funds")
    .addField("To account", `0x${address}`, true)
    .addField("Amount sent", `${params.TOKEN_COUNT} DEV`, true)
    .addField("Current account balance", `${accountBalance / 10n ** TOKEN_DECIMAL} DEV`)
    .setFooter(
      `Funds transactions are limited to once every ${params.FAUCET_SEND_INTERVAL} hour(s)`
    );

  msg.channel.send(fundsTransactionEmbed);
}

/**
 * Action for the bot for the pattern "!balance <h160_addr>", that
 * checks the balance of the indicated account.
 * @param {Message} msg Received discord message object
 * @param {string} messageContent Content of the message
 */
export async function botActionBalance(web3Api: Web3, msg: Message, messageContent: string) {
  let address = messageContent.slice("!balance".length).trim();
  if (address.startsWith("0x")) {
    address = address.slice(2);
  }

  // check address and send alert msg and return if bad formatted
  if (!checkH160AddressIsCorrect(address, msg)) return;

  const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));

  const balanceEmbed = new MessageEmbed()
    .setColor(EMBED_COLOR_CORRECT)
    .setTitle("Account Balance")
    .addField("Account", `0x${address}`, true)
    .addField("Balance", `${accountBalance / 10n ** TOKEN_DECIMAL} DEV`, true);

  msg.channel.send(balanceEmbed);
}

/**
 * Checks the balance of the bot is over a certain threshold, alerting
 * if that's the case on Slack
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 */
export async function balanceCheck(web3Api: Web3) {
  let balance = 0n;
  const alert_threshold = params.BALANCE_ALERT_THRESHOLD * 10n ** TOKEN_DECIMAL;

  while (true) {
    try {
      // Get balance using web3 API
      balance = BigInt(await web3Api.eth.getBalance(`0x${params.ACCOUNT_ID}`));

      // Check if balance is below the threshold and alert if so
      if (balance < alert_threshold) {
        await sendSlackNotification(
          params.SLACK_WEBHOOK,
          params.ACCOUNT_ID,
          balance / 10n ** TOKEN_DECIMAL,
          params.TOKEN_COUNT
        );
      }

      // Sleep for 10min
      await sleep(0, 10);
    } catch (e) {
      // In case of error, log and sleep for 3sec before retrying
      console.log(new Date().toISOString(), "ERROR", e.stack || e);
      await sleep(3);
    }
  }
}
