// This script is expected to run against a parachain network (using launch.ts script)

import { ALITH_PRIVATE_KEY } from "../utils/constants";
import { Keyring } from "@polkadot/api";
import { TransactionReceipt } from "web3-core";

import yargs from "yargs";
import { getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import Web3 from "web3";
import { customRequest, importAccount, init } from "../init-web3";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    "eth-url": {
      type: "string",
      description: "RPC url for Eth API",
      demandOption: true,
    },
    from: {
      type: "string",
      description: "Private key to transfer from",
      conflicts: ["to"],
    },
    count: {
      type: "number",
      default: 1000,
      description: "Number of accounts",
      demandOption: true,
    },
    loop: {
      type: "number",
      default: 1000,
      description: "How many loop in the contract",
      demandOption: true,
    },
  })
  .check(function (argv) {
    if (!argv.from && !argv.to) {
      argv.from = ALITH_PRIVATE_KEY;
    }
    return true;
  }).argv;

const sendTransfer = async (web3: Web3, from: any, nonce: number) => {
  const tx = await web3.eth.accounts.signTransaction(
    {
      from: from.address,
      to: "0x17e9bfd55118c142e15d36200dcdabb3aa5a0ac9",
      gasPrice: web3.utils.toWei("0.001", "ether"),
      gas: 1000000,
      value: web3.utils.toWei("1", "Gwei"),
      nonce: nonce++,
    },
    from.privateKey
  );

  const result = await customRequest("eth_sendRawTransaction", [tx.rawTransaction]);
  if (result.error) {
    console.error(result.error);
    throw new Error(`Error calling contract!`);
  }

  // console.log(`Transaction for Loop count ${loopCount} sent: ${tx.transactionHash}`);
  const startTime = Date.now();
  while (Date.now() - startTime < 60000) {
    let rcpt: TransactionReceipt = await web3.eth.getTransactionReceipt(tx.transactionHash);
    if (rcpt) {
      //console.log(`Loop count ${loopCount} - block #${rcpt.blockNumber} (${rcpt.blockHash})`);
      return;
    }
    await new Promise((resolve) => {
      setTimeout(resolve, 2000);
    });
  }
  throw new Error("Failed to verify contract call (timeout)");
};

const main = async () => {
  const web3 = init(argv["eth-url"]);
  const polkadotApi = await getMonitoredApiFor(argv);
  const keyring = new Keyring({ type: "ethereum" });

  const fromAccount = await keyring.addFromUri(argv.from);
  const deployer = importAccount(argv.from);

  let fromNonce = (await polkadotApi.rpc.system.accountNextIndex(fromAccount.address)).toNumber();

  // We need to multiple the float first to then convert to BigInt,
  // 1000000 should be enough
  console.log(`Sending ${argv.count} evm transfers...`);
  await Promise.all(
    new Array(argv.count).fill(0).map(() => {
      return sendTransfer(web3, deployer, fromNonce++);
    })
  );

  await polkadotApi.disconnect();
  await (web3.currentProvider as any).disconnect();
  console.log(`Finished`);
};

main();
