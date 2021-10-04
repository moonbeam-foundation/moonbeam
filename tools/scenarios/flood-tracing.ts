// This script is expected to run against a parachain network (using launch.ts script)

import yargs from "yargs";
import { getMonitoredApiFor, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { customRequest, init } from "../init-web3";

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
    transactions: {
      alias: "txs",
      type: "array",
      default: [
        "0x3fdfd2ad435af43537de5dd09e788a81a689c3e6d718a8afbcdaf2095171a9f7", // #581189 - 701
        "0xae5fa460151369280f54668b223453571833ee653392f1f00a83cd7947eecf81", // #455109 - 600
        "0xf5993cda3c5bc72fb6dd876ca5e6377ccf81d28d8f4b300a2c7a003de86baeff", // #430445 - 500
      ],
      description: "Transaction hash",
    },
  }).argv;

const callTracing = async (transaction: string) => {
  const start = Date.now();
  const result = await customRequest("debug_traceTransaction", [transaction]);
  if (result.error) {
    console.error(result.error);
    throw new Error(`Error calling tracing!`);
  }
  console.log(
    `${transaction}: ${(Date.now() - start).toString().padStart(6)}ms - ${result.result.length}`
  );
};

const main = async () => {
  const web3 = init(argv["eth-url"]);
  const polkadotApi = await getMonitoredApiFor(argv);

  while (true) {
    await Promise.all(
      argv.transactions.map((tx) => {
        return callTracing(tx);
      })
    );
  }
};

main();
