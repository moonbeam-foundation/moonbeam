
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
