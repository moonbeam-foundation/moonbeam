// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { typesBundle } from "moonbeam-types-bundle";

const PROVIDERS = [
  //   "wss://wss.moonriver.moonbeam.network",
  "wss://moonriver.api.onfinality.io/public-ws",
  "ws://localhost:56992",
  "ws://rpcnode1.moonriver.moonbeam.network:9944",
];

const hashString = (hash: string) => {
  return `${hash.toString().substring(0, 7)}...${hash
    .toString()
    .substring(hash.toString().length - 4)}`;
};

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    from: {
      type: "number",
      description: "from block number (included)",
      demandOption: true,
    },
    to: {
      type: "number",
      description: "to block number (included)",
    },
  }).argv;

const main = async () => {
  const nodes = await Promise.all(
    PROVIDERS.map(async (endpoint) => {
      const node = {
        endpoint,
        api: await ApiPromise.create({
          provider: new WsProvider(endpoint),
          typesBundle: typesBundle as any,
        }),
      };
      try {
        await node.api.isReadyOrError;
      } catch (err) {
        console.log(`✘ Couldn't set up API, is the endpoint up?. ${err.toString()}`);
        process.exit(1);
      }
      return node;
    })
  );

  const toBlockNumber =
    argv.to || (await nodes[0].api.rpc.chain.getBlock()).block.header.number.toNumber();
  const fromBlockNumber = argv.from;

  const concurrency = 10;
  for (let blockLoop = fromBlockNumber; blockLoop < toBlockNumber; blockLoop += concurrency) {
    if (blockLoop % 2000 == fromBlockNumber % 2000) {
      console.log(`${blockLoop}...`);
    }
    await Promise.all(
      new Array(concurrency).fill(0).map(async (_, i) => {
        const blockNumber = blockLoop + i;
        const data = await Promise.all(
          nodes.map(async ({ api }) => {
            let hash = "";
            let block = null;
            try {
              hash = (await api.rpc.chain.getBlockHash(blockNumber)).toString();
              block = (await api.rpc.chain.getBlock(hash)).block;
            } catch (e) {}
            return {
              hash,
              block,
            };
          })
        );

        const allEqualHash =
          data[0].hash && data[0].hash.length > 0 && data.every((v) => v.hash === data[0].hash);
        const allEqualBlock =
          data[0].block &&
          data[0].block.header.hash.toString().length > 0 &&
          data.every((v, index) => {
            try {
              return v.block.header.hash.toString() === data[0].block.header.hash.toString();
            } catch (e) {
              const { endpoint } = nodes[index];
              console.log(`${endpoint.padStart(50, " ")}: failed to retrieve header`);
            }
            return false;
          });

        if (!allEqualHash || !allEqualBlock) {
          for (const index in nodes) {
            const { endpoint } = nodes[index];
            const { hash, block } = data[index];
            console.log(
              `${endpoint.padStart(50, " ")}: block ${blockNumber} (hash: ${
                hash ? hashString(hash) : "".padStart(9, " ")
              }, parent: ${block ? hashString(block.header.parentHash) : "".padStart(9, " ")})`
            );
          }
          process.exit(1);
        }
      })
    );
  }
};

main();
