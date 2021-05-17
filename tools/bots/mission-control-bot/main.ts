import { Client, Message } from "discord.js";
import Web3 from "web3";

import { UserToTimestamp, BalanceCheck } from "./types";
import { params } from "./constants";
import { botActionBalance, botActionFaucetSend, balanceCheck } from "./actions";

async function onReceiveMessage(msg: Message) {
  const authorId = msg?.author?.id;
  const messageContent = msg?.content;
  const channelId = msg?.channel?.id;
  const acceptedChannels = [params.DISCORD_CHANNEL, params.TESTS_DISCORD_CHANNEL];

  if (!messageContent || !authorId || !acceptedChannels.includes(channelId)) {
    return;
  }

  if (messageContent.startsWith("!faucet send")) {
    await botActionFaucetSend(
      web3Api,
      msg,
      authorId,
      messageContent,
      receivers,
      lastBalanceCheck,
      pendingQueue
    );
  } else if (messageContent.startsWith("!balance")) {
    await botActionBalance(web3Api, msg, messageContent);
  }
}

// Check params are correctly defined
Object.keys(params).forEach((param) => {
  if (!params[param]) {
    console.log(`Missing ${param} env variables`);
    process.exit(1);
  }
});

const receivers: UserToTimestamp = {};
const pendingQueue: string[] = [];
const lastBalanceCheck: BalanceCheck = {
  timestamp: 0,
  balance: BigInt(0),
};

console.log(`Starting bot...`);
const client: Client = new Client();

console.log(`Connecting web3 to ${params.RPC_URL}...`);
const web3Api = new Web3(params.RPC_URL);

// Prompt when logged in
client.on("ready", () => console.log(`Logged in as ${client.user.tag}!`));

// Bind message event to custom listener
client.on("message", async (msg) => {
  try {
    await onReceiveMessage(msg);
  } catch (e) {
    console.log(new Date().toISOString(), "ERROR", e.stack || e);
  }
});

// Start balance checker
balanceCheck(web3Api);

// Perform login and listen for new events
client.login(params.DISCORD_TOKEN);
