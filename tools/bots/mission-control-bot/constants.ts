export const TOKEN_DECIMAL = 18n;
export const EMBED_COLOR_CORRECT = 0x642f95;
export const EMBED_COLOR_ERROR = 0xc0392b;

export const params = {
  // Discord app information
  DISCORD_TOKEN: process.env.DISCORD_TOKEN,
  DISCORD_CHANNEL: process.env.DISCORD_CHANNEL,

  // Items for monitoring
  TESTS_DISCORD_CHANNEL: process.env.TESTS_DISCORD_CHANNEL,
  NOT_LIMITED_USERS: JSON.parse(process.env.NOT_LIMITED_USERS || "false"),
  BALANCE_MONITOR_INTERVAL: parseInt(process.env.BALANCE_MONITOR_INTERVAL || "3"), // minutes

  // Slack app information
  SLACK_WEBHOOK: process.env.SLACK_WEBHOOK,

  // Web3 RPC access
  RPC_URL: process.env.RPC_URL,
  ACCOUNT_ID: process.env.ACCOUNT_ID,
  ACCOUNT_KEY: process.env.ACCOUNT_KEY,
  WORKERS_MNEMONIC: process.env.WORKERS_MNEMONIC,

  // Token distribution
  TOKEN_COUNT: BigInt(process.env.TOKEN_COUNT || 10), // DEV
  WORKERS_COUNT: parseInt(process.env.WORKERS_COUNT || "10"),
  WORKER_MIN_BALANCE: BigInt(process.env.WORKER_MIN_BALANCE || 200), // DEV
  GAS_PRICE: BigInt(process.env.GAS_PRICE || 1), // wei
  FAUCET_SEND_INTERVAL: parseInt(process.env.FAUCET_SEND_INTERVAL || "24"), // hours
  BALANCE_ALERT_THRESHOLD: BigInt(process.env.BALANCE_ALERT_THRESHOLD || 800), // DEV
};
