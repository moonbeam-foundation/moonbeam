import { JsonRpcResponse } from "web3-core-helpers";
import Web3 from "web3";
import pMap from "p-map";

function sliceIntoChunks(arr, chunkSize) {
  const res = [];
  for (let i = 0; i < arr.length; i += chunkSize) {
    const chunk = arr.slice(i, i + chunkSize);
    res.push(chunk);
  }
  return res;
}

export const PromiseConcurrent = <T, R>(
  concurrency: number,
  mapper: (item: T, index?: number) => Promise<R> | R,
  list: T[]
): Promise<R[]> => pMap(list, mapper, { concurrency: concurrency });

export async function customWeb3Request(web3: Web3, method: string, params: any[]) {
  return new Promise<JsonRpcResponse>((resolve, reject) => {
    (web3.currentProvider as any).send(
      {
        jsonrpc: "2.0",
        id: 1,
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
      // address: "0x9b400d3a8a8d920d1ef4674095c354c9c3f929a8",
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
  const startBlock = 280_000;
  const range = 2000;
  const concurrent = 4;
  const totalReq = 140;

  const httpProviderUrl = process.argv[process.argv.length - 1].startsWith("http")
    ? process.argv[process.argv.length - 1]
    : `http://35.229.103.120:9933`;
  console.log(`Using ${httpProviderUrl}`);
  const web3 = new Web3(httpProviderUrl);

  const startTime = Date.now();
  const requests = new Array(totalReq).fill(0).map((_, i) => startBlock - range * i);

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
