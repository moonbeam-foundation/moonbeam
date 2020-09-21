import { Client, MessageEmbed } from "discord.js";
import Web3 from "web3";


const TOKEN_DECIMAL = 18n;
const FAUCET_SEND_INTERVAL = 1; // hours
const EMBED_COLOR_CORRECT = 0x642f95;
const EMBED_COLOR_ERROR = 0xc0392b;

const params = {
	// Discord app information
	DISCORD_TOKEN: process.env.DISCORD_TOKEN,
	DISCORD_CHANNEL: process.env.DISCORD_CHANNEL,

	// Web3 RPC access
	RPC_URL: process.env.RPC_URL,
	ACCOUNT_KEY: process.env.ACCOUNT_KEY,

	// Token distribution
	TOKEN_COUNT: BigInt(process.env.TOKEN_COUNT || 10),
}

Object.keys(params).forEach(param => {
	if (!params[param]) {
		console.log(`Missing ${param} env variables`);
		process.exit(1);
	}
})

const web3Api = new Web3(params.RPC_URL);

console.log(`Starting bot...`);
console.log(`Connecting web3 to ${params.RPC_URL}...`);

const client: Client = new Client();
const receivers: { [author: string]: number } = {};

client.on("ready", () => {
	console.log(`Logged in as ${client.user.tag}!`);
});

/**
 * Returns the approximated remaining time until being able to request tokens again.
 * @param {Date} lastTokenRequestMoment
 */
const nextAvailableToken = (lastTokenRequestMoment) => {
	// how many ms there are in minutes/hours
	const msPerMinute = 60 * 1000;
	const msPerHour = msPerMinute * 60;
	
	// when the author of the message will be able to request more tokens
	const availableAt = lastTokenRequestMoment + (FAUCET_SEND_INTERVAL * msPerHour);
	// remaining time until able to request more tokens
	let remain = availableAt - Date.now();

	if (remain < msPerMinute) {
		return `${Math.round(remain / 1000)} second(s)`;
	}
	else if (remain < msPerHour) {
		return `${Math.round(remain / msPerMinute)} minute(s)`;
	}
	else {
		return `${Math.round(remain / msPerHour)} hour(s)`;
	}
}


const onReceiveMessage = async (msg) => {
	const authorId = msg && msg.author && msg.author.id;
	const messageContent = msg && msg.content;
	const channelId = msg && msg.channel && msg.channel.id;

	if (!messageContent || !authorId || channelId != params.DISCORD_CHANNEL) {
		return;
	}

	if (messageContent.startsWith("!faucet send")) {
		if (receivers[authorId] > Date.now() - 3600 * 1000) {
			const errorEmbed = new MessageEmbed()
				.setColor(EMBED_COLOR_ERROR)
				.setTitle(`You already received tokens!`)
				.addField("Remaining time", `You still need to wait ${nextAvailableToken(receivers[authorId])} to receive more tokens`)
				.setFooter("Funds transactions are limited to once per hour");

			msg.channel.send(errorEmbed);
			return;
		}
		let address = messageContent.slice("!faucet send".length).trim();
		if (address.startsWith("0x")) {
			address = address.slice(2);
		}
		if (address.length != 40) {
			const errorEmbed = new MessageEmbed()
				.setColor(EMBED_COLOR_ERROR)
				.setTitle("Invalid address")
				.setFooter("Addresses must follow the H160 address format");

			msg.channel.send(errorEmbed);
			return;
		}
		receivers[authorId] = Date.now();

		await web3Api.eth.sendSignedTransaction(
			(
				await web3Api.eth.accounts.signTransaction(
					{
						value: `${params.TOKEN_COUNT * (10n**TOKEN_DECIMAL)}`,
						gasPrice: "0x01",
						gas: "0x21000",
						to: `0x${address}`,
					},
					params.ACCOUNT_KEY
				)
			).rawTransaction
		);
		const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));

		const fundsTransactionEmbed = new MessageEmbed()
			.setColor(EMBED_COLOR_CORRECT)
			.setTitle("Transaction of funds")
			.addField("To account", `0x${address}`, true)
			.addField("Amount sent", `${params.TOKEN_COUNT} DEV`, true)
			.addField("Current account balance", `${accountBalance / (10n ** TOKEN_DECIMAL)} DEV`)
			.setFooter("Funds transactions are limited to once per hour");

		msg.channel.send(fundsTransactionEmbed);
	}
	if (messageContent.startsWith("!balance")) {
		let address = messageContent.slice("!balance".length).trim();
		if (address.startsWith("0x")) {
			address = address.slice(2);
		}
		if (address.length != 40) {
			const errorEmbed = new MessageEmbed()
				.setColor(EMBED_COLOR_ERROR)
				.setTitle("Invalid address")
				.setFooter("Addresses must follow the H160 address format");

			msg.channel.send(errorEmbed);
			return;
		}
		const accountBalance = BigInt(await web3Api.eth.getBalance(`0x${address}`));

		const balanceEmbed = new MessageEmbed()
			.setColor(EMBED_COLOR_CORRECT)
			.setTitle("Account Balance")
			.addField("Account", `0x${address}`, true)
			.addField("Balance", `${accountBalance / (10n ** TOKEN_DECIMAL)} DEV`, true);

		msg.channel.send(balanceEmbed);
	}
};

client.on("message", onReceiveMessage);

client.login(params.DISCORD_TOKEN);
