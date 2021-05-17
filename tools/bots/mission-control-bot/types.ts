// Custom defined types for the mission-control-bot

export type UserToTimestamp = {
  [author: string]: number;
};

export type BalanceCheck = {
  timestamp: number;
  balance: bigint;
}