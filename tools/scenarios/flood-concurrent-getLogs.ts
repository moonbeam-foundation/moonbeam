import { JsonRpcResponse } from "web3-core-helpers";
import Web3 from "web3";
import pMap from "p-map";
import yargs from "yargs";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    "eth-url": {
      type: "string",
      description: "RPC url for Eth API",
      demandOption: true,
    },
    from: {
      type: "number",
      description: "Block number to start with",
    },
    count: {
      type: "number",
      default: 100,
      description: "Total number of requests",
    },
    range: {
      type: "number",
      default: 2000,
      description: "Number of blocks to query",
    },
    concurrency: {
      type: "number",
      default: 4,
      demandOption: true,
      description: "Number of concurrent requests",
    },
  }).argv;

export const PromiseConcurrent = <T, R>(
  concurrency: number,
  mapper: (item: T, index?: number) => Promise<R> | R,
  list: T[]
): Promise<R[]> => pMap(list, mapper, { concurrency: concurrency });

let reqId = 10000;
export async function customWeb3Request(web3: Web3, method: string, params: any[]) {
  return new Promise<JsonRpcResponse>((resolve, reject) => {
    (web3.currentProvider as any).send(
      {
        jsonrpc: "2.0",
        id: reqId++,
        method,
        params,
      },
      (error: Error | null, result?: JsonRpcResponse) => {
        if (error) {
          reject(
            `Failed to send custom request (${method} (${params.join(",")})): ${
              error.message || error.toString()
            }`
          );
        }
        resolve(result);
      }
    );
  });
}

const queryRange = async (web3: Web3, startBlock, range, tag) => {
  const start = Date.now();
  const { result } = await customWeb3Request(web3, "eth_getLogs", [
    {
      fromBlock: web3.utils.numberToHex(startBlock),
      toBlock: web3.utils.numberToHex(startBlock + range - 1),
      topics: [],
      address: "0x9b400d3a8a8d920d1ef4674095c354c9c3f929a8",
    },
  ]);
  const end = Date.now();
  console.log(
    `[${tag}] ${startBlock}-${startBlock + range - 1} Took: ${end - start} ms: ${
      result?.length
    } logs`
  );
  return end - start;
};

const main = async () => {
  const from = argv.from || 0;
  const range = argv.range;
  const concurrent = argv.concurrency;
  const totalReq = argv.count;

  const httpProviderUrl = argv["eth-url"];
  console.log(`Using ${httpProviderUrl}`);
  const web3 = new Web3(httpProviderUrl);

  const startTime = Date.now();
  const requests = new Array(totalReq).fill(0).map((_, i) => from + range * i);

  const allTimes = await PromiseConcurrent(
    concurrent,
    (req, i) => queryRange(web3, req, range, `req ${i}`),
    requests
  );

  const endTime = Date.now();

  console.log(`Total time: ${endTime - startTime} (${totalReq} req)`);
  console.log(
    `Avg time: ${allTimes.reduce((v, p) => p + v, 0) / totalReq} vs ${
      (endTime - startTime) / totalReq
    }`
  );
};

main();
