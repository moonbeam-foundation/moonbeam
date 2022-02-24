import { MessageEmbed, Message } from "discord.js";
import Web3 from "web3";
import { TransactionReceipt } from "web3-core";
import PromiEvent from "web3-core-promievent";

import { Receivers, WorkerAccount, FundsRequest } from "./types";
import { TOKEN_DECIMAL, EMBED_COLOR_CORRECT, EMBED_COLOR_ERROR, params } from "./constants";
import { sendSlackNotification, nextAvailableToken, sleep } from "./utils";

/**
 * Action for the bot for the pattern "!faucet send <h160_addr>", that
 * sends funds to the indicated account.
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 * @param msg Received discord message object
 * @param discordUserId Author ID of the message
 * @param messageContent Content of the message
 * @param discordUserReceivers Map with the timestamp of the last received request of a user
 * @param pendingQueue Queue of tasks
 */
export async function botActionFaucetSend(
  web3Api: Web3,
  msg: Message,
  discordUserId: string,
  messageContent: string,
  discordUserReceivers: Receivers,
  addressReceivers: Receivers,
  pendingQueue: FundsRequest[]
) {
  // get address from message
  let address = messageContent.slice("!faucet send".length).trim();
  if (!address.startsWith("0x")) address = `0x${address}`;

  // check address and send alert msg and return if bad formatted
  if (!web3Api.utils.isAddress(address)) {
    const errorEmbed = new MessageEmbed()
      .setColor(EMBED_COLOR_ERROR)
      .setTitle("Invalid address")
      .setFooter("Addresses must follow the H160 address format");

    // send message to channel
    msg.channel.send(errorEmbed);
    // exit
    return;
  }

  // set default value of lastReceived at 0
  if (!discordUserReceivers[discordUserId]) discordUserReceivers[discordUserId] = 0;
  if (!addressReceivers[address]) addressReceivers[address] = 0;

  const lastPermittedClaimDate = Date.now() - params.FAUCET_SEND_INTERVAL * 3600 * 1000;

  const [reason, canReceiveTokensAgain] = params.NOT_LIMITED_USERS.includes(discordUserId)
    ? [null, true]
    : discordUserReceivers[discordUserId] > lastPermittedClaimDate
    ? ["user", false]
    : addressReceivers[address] > lastPermittedClaimDate
    ? ["address", false]
    : [null, true];

  if (!canReceiveTokensAgain) {
    const errorEmbed = new MessageEmbed();

    // add user or address that has been already used
    if (reason === "address") {
      errorEmbed.addField("Address", address);
    } else {
      errorEmbed.addField("User", msg?.author?.username);
    }
    // add rest of fields
    errorEmbed
      .setColor(EMBED_COLOR_ERROR)
      .setTitle(`You already received tokens!`)
      .addField(
        "Remaining time",
        `You still need to wait ${nextAvailableToken(
          reason === "address" ? addressReceivers[address] : discordUserReceivers[discordUserId],
          params.FAUCET_SEND_INTERVAL
        )} to receive more tokens`
      )
      .setFooter(
        `Funds transactions are limited to once every ${params.FAUCET_SEND_INTERVAL} hour(s)`
      );

    msg.channel.send(errorEmbed);
    return;
  }

  // create funds request
  const fundsRequest: FundsRequest = {
    discordUser: msg?.author?.username,
    discordChannel: msg.channel,
    address: address,
    prevTimestampAddress: JSON.parse(JSON.stringify(addressReceivers[address])),
    prevTimestampUser: JSON.parse(JSON.stringify(discordUserReceivers[discordUserId])),
  };

  // update the last time user/address requested funds
  let now = Date.now();
  addressReceivers[address] = now;
  discordUserReceivers[discordUserId] = now;

  // push request to the queue to be resolved later
  pendingQueue.push(fundsRequest);
}

/**
 * Action for the bot for the pattern "!balance <h160_addr>", that
 * checks the balance of the indicated account.
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 * @param msg Received discord message object
 * @param messageContent Content of the message
 */
export async function botActionBalance(web3Api: Web3, msg: Message, messageContent: string) {
  let address = messageContent.slice("!balance".length).trim();
  if (!address.startsWith("0x")) address = `0x${address}`;

  // check address and send alert msg and return if bad formatted
  if (!web3Api.utils.isAddress(address)) {
    const errorEmbed = new MessageEmbed()
      .setColor(EMBED_COLOR_ERROR)
      .setTitle("Invalid address")
      .setFooter("Addresses must follow the H160 address format");

    // send message to channel
    msg.channel.send(errorEmbed);
    // exit
    return;
  }

  const accountBalance = BigInt(await web3Api.eth.getBalance(address));

  const balanceEmbed = new MessageEmbed()
    .setColor(EMBED_COLOR_CORRECT)
    .setTitle("Account Balance")
    .addField("Account", address, true)
    .addField("Balance", `${accountBalance / 10n ** TOKEN_DECIMAL} DEV`, true);

  msg.channel.send(balanceEmbed);
}

/**
 * Checks the balance of the bot is over a certain threshold, alerting
 * if that's the case on Slack
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 * @param workers List of worker accounts to fund if they're below the min balance
 */
export async function balanceMonitorCheck(web3Api: Web3, workers: WorkerAccount[]) {
  const alertThreshold = params.BALANCE_ALERT_THRESHOLD * 10n ** TOKEN_DECIMAL;
  const workerMinBalance = params.WORKER_MIN_BALANCE * 10n ** TOKEN_DECIMAL;
  const overBalance = 5n * params.TOKEN_COUNT * 10n ** TOKEN_DECIMAL;

  while (true) {
    let start = Date.now();

    try {
      for (const worker of workers) {
        // Get worker balance using web3 API
        let workerBalance = BigInt(await web3Api.eth.getBalance(worker.address));
        let difference = workerMinBalance - workerBalance;

        // If the balance is below the threslhold, fund it with main account
        if (difference > 0n) {
          let waitConfirmation = new Promise(async (resolve) => {
            await web3Api.eth
              .sendSignedTransaction(
                (
                  await web3Api.eth.accounts.signTransaction(
                    {
                      value: (difference + overBalance).toString(),
                      gasPrice: params.GAS_PRICE.toString(),
                      gas: "21000",
                      to: worker.address,
                    },
                    params.ACCOUNT_KEY
                  )
                ).rawTransaction
              )
              .on("confirmation", (nbrOfBlocks, receipt) => {
                // wait for 1 block confirmation to avoid Low Priority failure on next transaction
                if (nbrOfBlocks >= 1) {
                  resolve(receipt);
                }
              });
          });

          await waitConfirmation;
        }
      }
    } catch (error) {
      // In case of error, just log it and continue
      console.log(new Date().toISOString(), "ERROR_1_0", error.stack || error);
    }

    try {
      // Get main account balance using web3 API
      let mainBalance = BigInt(await web3Api.eth.getBalance(params.ACCOUNT_ID));

      // Check if balance is below the threshold and alert if so
      if (mainBalance < alertThreshold) {
        await sendSlackNotification(
          params.SLACK_WEBHOOK,
          params.ACCOUNT_ID,
          mainBalance / 10n ** TOKEN_DECIMAL
        );
      }
    } catch (error) {
      // In case of error, log and sleep for 3sec before retrying
      console.log(new Date().toISOString(), "ERROR_1_1", error.stack || error);
    }

    // total elapsed/remaining in minutes
    let elapsed = (Date.now() - start) / 1000 / 60;
    let remaining = params.BALANCE_MONITOR_INTERVAL - elapsed;

    // sleep what is necessary to do the check every params.BALANCE_MONITOR_INTERVAL minutes
    await sleep(0, remaining > 0 ? remaining : 0);
  }
}

/**
 * Resolves a total of "workers.length" funds requests every block (12sec)
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 * @param workers Array of worker accounts that will be used to sign the transfer txs
 * @param pendingQueue Queue of tasks
 * @param discordUserReceivers Map with the timestamp of the last received request of
 * a discord user
 * @param addressReceivers Map with the timestamp of the last received request of an
 * address
 */
export async function fundRequestsResolver(
  web3Api: Web3,
  workers: WorkerAccount[],
  pendingQueue: FundsRequest[],
  discordUserReceivers: Receivers,
  addressReceivers: Receivers
) {
  while (true) {
    let start = Date.now();

    try {
      await resolveFundsRequests(
        web3Api,
        workers,
        pendingQueue,
        discordUserReceivers,
        addressReceivers
      );
    } catch (error) {
      console.log(new Date().toISOString(), "ERROR_2_0", error.stack || error);
    }

    // total elapsed/remaining in seconds
    let elapsed = (Date.now() - start) / 1000;
    let remaining = 12 - elapsed;

    // sleep what is necessary to resolve every 12 seconds (block time)
    await sleep(remaining > 0 ? remaining : 0);
  }
}

/**
 * Non-exported function that takes fundsRequests from the pendingQueue and
 * resolves them with a series of different worker addresses to avoid a
 * nonce conflict.
 * @param web3Api Instance of the web3 API connected to the chain endpoint
 * @param workers Array of worker accounts that will be used to sign the transfer txs
 * @param pendingQueue Queue of tasks
 * @param discordUserReceivers Map with the timestamp of the last received request of
 * a discord user
 * @param addressReceivers Map with the timestamp of the last received request of an
 * address
 */
async function resolveFundsRequests(
  web3Api: Web3,
  workers: WorkerAccount[],
  pendingQueue: FundsRequest[],
  discordUserReceivers: Receivers,
  addressReceivers: Receivers
) {
  type TransferReceipts = {
    request: FundsRequest;
    receiptPromise: PromiEvent<TransactionReceipt>;
  };

  let transferReceipts: TransferReceipts[] = [];

  for (const worker of workers) {
    const workerBalance = BigInt(await web3Api.eth.getBalance(worker.address));
    const requiredBalance = params.TOKEN_COUNT * 10n ** TOKEN_DECIMAL + 21000n;

    // if the worker doesn't have enough balance, skip before
    // consuming a task from the queue. This way workers
    // at the end of the list, which are not used that often,
    // will resolve the requests until this worker is funded again
    if (workerBalance < requiredBalance) continue;

    const fundsRequest = pendingQueue.pop();

    // if the fr is undefined, it means that the queue is empty
    if (!fundsRequest) continue;

    let waitForReceipt = new Promise(async (resolve) => {
      await web3Api.eth
        .sendSignedTransaction(
          (
            await web3Api.eth.accounts.signTransaction(
              {
                value: `${params.TOKEN_COUNT * 10n ** TOKEN_DECIMAL}`,
                gasPrice: params.GAS_PRICE.toString(),
                gas: "21000",
                to: fundsRequest.address,
              },
              worker.privateKey
            )
          ).rawTransaction
        )
        .on("confirmation", (nbrOfBlocks, receipt) => {
          // wait for 1 block confirmation to avoid Low Priority failure on next transaction
          if (nbrOfBlocks >= 1) {
            resolve(receipt);
          }
        });
    });

    transferReceipts.push({
      request: fundsRequest,
      receiptPromise: waitForReceipt,
    });
  }

  for (const tr of transferReceipts) {
    try {
      const receipt = await tr.receiptPromise;
      const accountBalance = BigInt(await web3Api.eth.getBalance(tr.request.address));

      const fundsTransactionEmbed = new MessageEmbed()
        .setColor(EMBED_COLOR_CORRECT)
        .setTitle("Transaction of funds")
        .addField("To account", tr.request.address, true)
        .addField("Amount sent", `${params.TOKEN_COUNT} DEV`, true)
        .addField("Transaction Hash", `${receipt.transactionHash}`, false)
        .addField("Current account balance", `${accountBalance / 10n ** TOKEN_DECIMAL} DEV`, false)
        .setFooter(
          `Funds transactions are limited to once every ${params.FAUCET_SEND_INTERVAL} hour(s)`
        );

      tr.request.discordChannel.send(fundsTransactionEmbed);
    } catch (error) {
      // rollback the update of user/address last funds request
      discordUserReceivers[tr.request.discordUser] = tr.request.prevTimestampUser;
      addressReceivers[tr.request.address] = tr.request.prevTimestampAddress;

      // alert in channel
      const errorEmbed = new MessageEmbed()
        .setColor(EMBED_COLOR_ERROR)
        .setTitle("Could not submit the transaction")
        .setFooter(
          "The transaction of funds could not be submitted. " +
            "Please, try requesting funds again or contact an admin."
        );

      // send message
      tr.request.discordChannel.send(errorEmbed);

      console.log(new Date().toISOString(), "ERROR_2_1", error.stack || error);
    }
  }
}
