// Custom defined types for the mission-control-bot

import { DMChannel, NewsChannel, TextChannel } from "discord.js";

export type Receivers = {
  [discordUser: string]: number;
};

export type WorkerAccount = {
  address: string;
  privateKey: string;
};

export type DiscordChannel = TextChannel | DMChannel | NewsChannel;

export type FundsRequest = {
  discordUser: string;
  discordChannel: DiscordChannel;
  address: string;
  prevTimestampUser: number;
  prevTimestampAddress: number;
};
