import https from "https";
import { hdkey } from "ethereumjs-wallet";

import { WorkerAccount } from "./types";

/**
 * Send notification to Slack using a webhook URL.
 *
 * @param webhookUrl Webhook to Slack app
 * @param accountId Account of the bot
 * @param account_balance Balance of the account in DEV
 */
export async function sendSlackNotification(
  webhookUrl: string,
  accountId: string,
  account_balance: bigint
) {
  // Message to send to Slack (JSON payload)
  const title = "Fund bot operational account";
  const message = "The account linked to the bot is running low on funds";

  const payload = {
    attachments: [
      {
        color: "warning",
        fallback:
          `${title}. ${message}\n` +
          `  * Current balance: ${account_balance.toString()} DEV\n` +
          `  * Desired balance: 10,000+ DEV\n` +
          `  * Fund the following account: ${accountId}`,
        title: title,
        text: message,
        fields: [
          {
            title: "Current balance",
            value: `${account_balance.toString()} DEV`,
            short: true,
          },
          {
            title: "Desired balance",
            value: `10,000+ DEV`,
            short: true,
          },
          {
            title: "Please, fund the following account",
            value: accountId,
            short: false,
          },
        ],
      },
    ],
  };

  // Options for the HTTP request (data is written later)
  const options = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": JSON.stringify(payload).length,
    },
  };

  // Promise to "await" until request has ended
  const completed_request = new Promise((resolve, reject) => {
    // Send request to Slack webhook
    const request = https
      .request(webhookUrl, options, (response) => {
        let data = "";

        response.on("data", (chunk) => {
          data += chunk;
        });

        response.on("end", () => {
          console.log("Received data from Slack webhook:", data);
          resolve(data);
        });
      })
      .on("error", (err) => {
        console.log("Error while sending Slack notification:", err.message);
        reject(err);
      });

    request.write(JSON.stringify(payload));
    request.end();
  });

  return await completed_request;
}

/**
 * Returns the approximated remaining time until being able to request tokens again.
 *
 * @param lastTokenRequestMoment Last moment in which the user requested funds
 * @param faucet_interval Interval between allowed fund requests to the bot
 */
export function nextAvailableToken(lastTokenRequestMoment: number, faucet_interval: number) {
  // how many ms there are in minutes/hours
  const msPerMinute = 60 * 1000;
  const msPerHour = msPerMinute * 60;

  // when the author of the message will be able to request more tokens
  const availableAt = lastTokenRequestMoment + faucet_interval * msPerHour;
  // remaining time until able to request more tokens
  let remain = availableAt - Date.now();

  if (remain < msPerMinute) {
    return `${Math.round(remain / 1000)} second(s)`;
  } else if (remain < msPerHour) {
    return `${Math.round(remain / msPerMinute)} minute(s)`;
  } else {
    return `${Math.round(remain / msPerHour)} hour(s)`;
  }
}

/**
 * Sleeps for a defined number of seconds and minutes (latter not required)
 *
 * @param seconds Number of seconds
 * @param minutes Number of minutes (not required)
 */
export async function sleep(seconds: number, minutes: number = 0) {
  let totalMs = minutes * 60 * 1000 + seconds * 1000;

  await new Promise((r) => setTimeout(r, totalMs));
}

/**
 * Returns an array with a range of numbers from 0 to length-1
 *
 * @param length Length of the resultant array
 */
export function range(length: number) {
  return [...Array(length).keys()];
}

/**
 * Derive the account of a worker from a mnemonic
 *
 * @param hdkeyGenerator The hdkey used to derive the account
 * @param index The index used for the path of the derivation
 * @returns The account of the worker with its address and private key
 */
export function deriveWorkerAccount(hdkeyGenerator: hdkey, index: number): WorkerAccount {
  const path = `m/44'/60'/0'/0/${index}`;
  const wallet = hdkeyGenerator.derivePath(path).getWallet();

  const worker: WorkerAccount = {
    address: wallet.getAddressString(),
    privateKey: wallet.getPrivateKeyString(),
  };

  return worker;
}
