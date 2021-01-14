import { Client, MessageEmbed, Message } from "discord.js";
import Web3 from "web3";
import https from "https";

const TOKEN_DECIMAL = 18n;
const EMBED_COLOR_CORRECT = 0x642f95;
const EMBED_COLOR_ERROR = 0xc0392b;
const SLACK_MSG_CONTENTS = `
{
  "blocks": [
    {
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "The account linked to the bot is running low on funds."
      }
    },
    {
      "type": "section",
      "fields": [
        {
          "type": "mrkdwn",
          "text": "*Account ID:*\n{{ account-fix-me }}"
        },
        {
          "type": "mrkdwn",
          "text": "*Current balance:*\n{{ balance-fix-me }} DEV"
        }
      ]
    }
  ]
}
`;

const params = {
  // Discord app information
  DISCORD_TOKEN: process.env.DISCORD_TOKEN,
  DISCORD_CHANNEL: process.env.DISCORD_CHANNEL,
  TESTS_DISCORD_CHANNEL: process.env.TESTS_DISCORD_CHANNEL,

  // Slack app information
  SLACK_WEBHOOK: process.env.SLACK_WEBHOOK,

  // Web3 RPC access
  RPC_URL: process.env.RPC_URL,
  ACCOUNT_ID: process.env.ACCOUNT_ID,
  ACCOUNT_KEY: process.env.ACCOUNT_KEY,

  // Token distribution
  TOKEN_COUNT: BigInt(process.env.TOKEN_COUNT || 10),
  FAUCET_SEND_INTERVAL: parseInt(process.env.FAUCET_SEND_INTERVAL || "1"), // hours
  BALANCE_ALERT_THRESHOLD: BigInt(process.env.BALANCE_ALERT_THRESHOLD || 100), // DEV
};

Object.keys(params).forEach((param) => {
  if (!params[param]) {
    console.log(`Missing ${param} env variables`);
    process.exit(1);
  }
});

const web3Api = new Web3(params.RPC_URL);

console.log(`Starting bot...`);
console.log(`Connecting web3 to ${params.RPC_URL}...`);

const client: Client = new Client();
const receivers: { [author: string]: number } = {};
const lastBalanceCheck = {
  timestamp: 0,
  balance: BigInt(0),
};

/**
 * Send notification to Slack using a webhook URL and the
 * message payload read from SLACK_MSG_CONTENT_FILEPATH.
 * @param {BigInt} account_balance Balance of the account in DEV
 */
const sendSlackNotification = async (account_balance: BigInt) => {
  // Message to send to Slack (JSON payload)
  const data = SLACK_MSG_CONTENTS.replace("{{ account-fix-me }}", params.ACCOUNT_ID).replace(
    "{{ balance-fix-me }}",
    account_balance.toString()
  );

  // Options for the HTTP request (data is written later)
  const options = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": data.length,
    },
  };

  // Promise to "await" until request has ended
  const completed_request = new Promise((resolve, reject) => {
    // Send request to Slack webhook
    const request = https
      .request(params.SLACK_WEBHOOK, options, (response) => {
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

    request.write(data);
    request.end();
  });

  return await completed_request;
};

/**
 * Returns the approximated remaining time until being able to request tokens again.
 * @param {number} lastTokenRequestMoment Last moment in which the user requested funds
 */
const nextAvailableToken = (lastTokenRequestMoment: number) => {
  // how many ms there are in minutes/hours
  const msPerMinute = 60 * 1000;
  const msPerHour = msPerMinute * 60;

  // when the author of the message will be able to request more tokens
  const availableAt = lastTokenRequestMoment + params.FAUCET_SEND_INTERVAL * msPerHour;
  // remaining time until able to request more tokens
  let remain = availableAt - Date.now();

  if (remain < msPerMinute) {
    return `${Math.round(remain / 1000)} second(s)`;
  } else if (remain < msPerHour) {
    return `${Math.round(remain / msPerMinute)} minute(s)`;
  } else {
    return `${Math.round(remain / msPerHour)} hour(s)`;
  }
};

/**
 * Checks that the address follows the H160 adress format
 * @param {string} address Address to check
 * @param {Message} msg Received discord message object
 */
const checkH160AddressIsCorrect = (address: string, msg: Message) => {
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
};

/**
 * Action for the bot for the pattern "!faucet send <h160_addr>", that
 * sends funds to the indicated account.
 * @param {Message} msg Received discord message object
 * @param {string} authorId Author ID of the message
 * @param {string} messageContent Content of the message
 */
const botActionFaucetSend = async (msg: Message, authorId: string, messageContent: string) => {
  if (receivers[authorId] > Date.now() - params.FAUCET_SEND_INTERVAL * 3600 * 1000) {
    const errorEmbed = new MessageEmbed()
      .setColor(EMBED_COLOR_ERROR)
      .setTitle(`You already received tokens!`)
      .addField(
        "Remaining time",
        `You still need to wait ${nextAvailableToken(receivers[authorId])} to receive more tokens`
      )
      .setFooter("Funds transactions are limited to once per hour");

    msg.channel.send(errorEmbed);
    return;
  }

  let address = messageContent.slice("!faucet send".length).trim();
  if (address.startsWith("0x")) {
    address = address.slice(2);
  }

  // check address and send alert msg and return if bad formatted
  if (!checkH160AddressIsCorrect(address, msg)) return;

  // update user last fund retrieval
  receivers[authorId] = Date.now();

  await web3Api.eth.sendSignedTransaction(
    (
      await web3Api.eth.accounts.signTransaction(
        {
          value: `${params.TOKEN_COUNT * 10n ** TOKEN_DECIMAL}`,
          gasPrice: "0x01",
          gas: "0x21000",
          to: `0x${address}`,
        },
        params.ACCOUNT_KEY
      )
    ).rawTransaction
  );
  const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));

  // Check balance every 10min (minimum interval, dependent on when the function is called)
  if (lastBalanceCheck.timestamp < Date.now() - 600 * 1000) {
    // Update cached info for last balance check
    lastBalanceCheck.balance = BigInt(await web3Api.eth.getBalance(`0x${params.ACCOUNT_ID}`));
    lastBalanceCheck.timestamp = Date.now();

    // If balance is low, send notification to Slack
    if (lastBalanceCheck.balance < params.BALANCE_ALERT_THRESHOLD * 10n ** TOKEN_DECIMAL) {
      await sendSlackNotification(lastBalanceCheck.balance / 10n ** TOKEN_DECIMAL);
    }
  }

  const fundsTransactionEmbed = new MessageEmbed()
    .setColor(EMBED_COLOR_CORRECT)
    .setTitle("Transaction of funds")
    .addField("To account", `0x${address}`, true)
    .addField("Amount sent", `${params.TOKEN_COUNT} DEV`, true)
    .addField("Current account balance", `${accountBalance / 10n ** TOKEN_DECIMAL} DEV`)
    .setFooter("Funds transactions are limited to once per hour");

  msg.channel.send(fundsTransactionEmbed);
};

/**
 * Action for the bot for the pattern "!balance <h160_addr>", that
 * checks the balance of the indicated account.
 * @param {Message} msg Received discord message object
 * @param {string} messageContent Content of the message
 */
const botActionBalance = async (msg: Message, messageContent: string) => {
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
};

const onReceiveMessage = async (msg: Message) => {
  const authorId = msg && msg.author && msg.author.id;
  const messageContent = msg && msg.content;
  const channelId = msg && msg.channel && msg.channel.id;

  if (
    !messageContent ||
    !authorId ||
    ![params.DISCORD_CHANNEL, params.TESTS_DISCORD_CHANNEL].includes(channelId)
  ) {
    return;
  }

  if (messageContent.startsWith("!faucet send")) {
    await botActionFaucetSend(msg, authorId, messageContent);
  } else if (messageContent.startsWith("!balance")) {
    await botActionBalance(msg, messageContent);
  }
};

// Prompt when logged in
client.on("ready", () => {
  console.log(`Logged in as ${client.user.tag}!`);
});

// Bind message event to custom listener
client.on("message", async (msg) => {
  try {
    await onReceiveMessage(msg);
  } catch (e) {
    console.log(new Date().toISOString(), "ERROR", e.stack || e);
  }
});

// Perform login and listen for new events
client.login(params.DISCORD_TOKEN);
