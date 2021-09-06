import yargs from "yargs";
import { getWeb3For, NETWORK_YARGS_OPTIONS } from "../utils/networks";
import { customWeb3Request } from "../utils/transactions";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    from: {
      type: "number",
      description: "from block number (included)",
      demandOption: true,
    },
    to: {
      type: "number",
      description: "to block number (included)",
    },
    account: {
      type: "string",
      description: "filter only specific nominator account",
    },
  }).argv;

const main = async () => {
  const web3 = await getWeb3For(argv.url || argv.network);

  const filteredAccount = argv.account?.toLowerCase() || null;

  const toBlockNumber = argv.to || (await web3.eth.getBlock("latest")).number;
  const fromBlockNumber = argv.from;
  console.log(
    `Using range #${fromBlockNumber}-#${toBlockNumber} (${toBlockNumber - fromBlockNumber + 1})`
  );

  console.log(`\n========= Retrieve transactions...`);

  const concurrency = 10;
  let txs = 0;
  for (let blockLoop = fromBlockNumber; blockLoop < toBlockNumber; blockLoop += concurrency) {
    if (blockLoop % 20 == fromBlockNumber % 20) {
      console.log(`${blockLoop}...(tx: ${txs})`);
    }
    await Promise.all(
      new Array(concurrency).fill(0).map(async (_, i) => {
        const blockNumber = blockLoop + i;
        const block = await web3.eth.getBlock(blockNumber);
        for (const transaction of block.transactions) {
          console.log(transaction);
          const data = await customWeb3Request(web3, "debug_traceTransaction", [transaction]);
          if (data.error) {
            console.log(`${blockNumber}: ${data.error}`);
          }
          txs++;
        }
      })
    );
  }
  await (web3.currentProvider as any).disconnect();
};

async function start() {
  try {
    await main();
  } catch (e) {
    console.error(e);
    process.exit(1);
  }
}

start();
