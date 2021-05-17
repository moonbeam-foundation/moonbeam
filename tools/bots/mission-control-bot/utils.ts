import https from "https";
import { MessageEmbed, Message } from "discord.js";

import { EMBED_COLOR_ERROR } from "./constants";

/**
 * Send notification to Slack using a webhook URL.
 * @param webhookUrl Webhook to Slack app
 * @param accountId Account of the bot
 * @param account_balance Balance of the account in DEV
 * @param tokens_issued Number of tokens issued each time
 */
export async function sendSlackNotification(
  webhookUrl: string,
  accountId: string,
  account_balance: bigint,
  tokens_issued: bigint
) {
  // Message to send to Slack (JSON payload)
  const title = "Fund bot operational account";
  const message = "The account linked to the bot is running low on funds.";
  const remainingAlerts = account_balance / tokens_issued;

  const payload = {
    attachments: [
      {
        color: "warning",
        fallback:
          `${title}. ${message}\n` +
          `  * Balance: ${account_balance.toString()} DEV\n` +
          `  * Alerts until failure: ${remainingAlerts.toString()}\n` +
          `  * Fund the following account: ${accountId}`,
        title: title,
        text: message,
        fields: [
          {
            title: "Balance",
            value: `${account_balance.toString()} DEV`,
            short: true,
          },
          {
            title: "Alerts until failure",
            value: `${remainingAlerts.toString()}`,
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
 * Checks that the address follows the H160 adress format
 * @param {string} address Address to check
 * @param {Message} msg Received discord message object
 */
export function checkH160AddressIsCorrect(address: string, msg: Message) {
  let addressIsCorrect = true;

  // slice address if defined in hexadecimal
  if (address.startsWith("0x")) {
    address = address.slice(2);
  }

  // check that address is 40 characters long
  if (address.length != 40) {
    addressIsCorrect = false;
  }

  // check that address only contains alphanumerical characters
  if (!address.match(/^[a-z0-9]+$/i)) {
    addressIsCorrect = false;
  }

  // resolve if address was not correct
  if (addressIsCorrect === false) {
    const errorEmbed = new MessageEmbed()
      .setColor(EMBED_COLOR_ERROR)
      .setTitle("Invalid address")
      .setFooter("Addresses must follow the H160 address format");

    // send message to channel
    msg.channel.send(errorEmbed);
  }

  return addressIsCorrect;
}

/**
 * Sleeps for a defined number of seconds and minutes (latter not required)
 * @param seconds Number of seconds
 * @param minutes Number of minutes (not required)
 */
export async function sleep(seconds: number, minutes: number = 0){
  let totalMs = (minutes * 60 * 1000) + (seconds * 1000)

  await new Promise((r) => setTimeout(r, totalMs));
}
