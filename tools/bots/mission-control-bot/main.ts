import { Client, Message } from "discord.js";
import Web3 from "web3";
import { mnemonicToSeedSync } from "bip39";
import { hdkey } from "ethereumjs-wallet";

import { Receivers, FundsRequest, WorkerAccount } from "./types";
import { params } from "./constants";
import {
  botActionBalance,
  botActionFaucetSend,
  balanceMonitorCheck,
  fundRequestsResolver,
} from "./actions";
import { deriveWorkerAccount, range } from "./utils";

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
      discordUserReceivers,
      addressReceivers,
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

const hdkeyGenerator = hdkey.fromMasterSeed(mnemonicToSeedSync(params.WORKERS_MNEMONIC));
const discordUserReceivers: Receivers = {};
const addressReceivers: Receivers = {};
const workers: WorkerAccount[] = [];
const pendingQueue: FundsRequest[] = [];

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
    console.log(new Date().toISOString(), "ERROR_0", e.stack || e);
  }
});

// Derive worker accounts
for (const index of range(params.WORKERS_COUNT)) {
  workers.push(deriveWorkerAccount(hdkeyGenerator, index));
}

// Start balance checker
console.log(`Starting bot balance monitor...`);
balanceMonitorCheck(web3Api, workers);

// Start resolver
console.log(`Starting funds requests resolver...`);
fundRequestsResolver(web3Api, workers, pendingQueue, discordUserReceivers, addressReceivers);

// Perform login and listen for new events
client.login(params.DISCORD_TOKEN);
