import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";

const fs = require("fs");
const URL = "http://localhost:9990";
const ERROR_FILE = "db/error.json";
const PROGRESS_FILE = "db/progress.json";

function lastProcessedBlock(): any {
  try {
    if (fs.existsSync(PROGRESS_FILE)) {
      let p = JSON.parse(fs.readFileSync(PROGRESS_FILE));
      if (p.hasOwnProperty("lastProcessedBlock")) {
        return p;
      } else {
        throw Error("Progress file is corrupted.");
      }
    } else {
      let p = {
        lastProcessedBlock: 0,
        ethTransactionsProcessed: 0,
      };
      fs.writeFileSync(PROGRESS_FILE, JSON.stringify(p));
      return p;
    }
  } catch (err) {
    throw err;
  }
}

function createErrorFile() {
  try {
    if (!fs.existsSync(ERROR_FILE)) {
      fs.writeFileSync(
        ERROR_FILE,
        JSON.stringify({
          errors: [],
        })
      );
    }
  } catch (err) {
    throw err;
  }
}

async function processBlock(web3: Web3, n: number): Promise<number> {
  // Get current block and iterate over its transaction hashes.
  let block = await web3.eth.getBlock(n);
  for (let txn of block.transactions) {
    let params = [txn];
    // Replay the current transaction.
    let req = new Promise<JsonRpcResponse>((resolve, reject) => {
      (web3.currentProvider as any).send(
        {
          jsonrpc: "2.0",
          id: 1,
          method: "debug_traceTransaction",
          params,
        },
        (error: Error | null, result?: JsonRpcResponse) => {
          // We are only interested in errors. Error in HTTP request.
          if (error) {
            let e = JSON.parse(fs.readFileSync(ERROR_FILE));
            let current = e.errors;
            current.push({
              block_number: n,
              txn: txn,
              error: error.message || error.toString(),
            });
            // Update error file.
            fs.writeFileSync(ERROR_FILE, JSON.stringify(current));
            reject(`Failed ((${params.join(",")})): ${error.message || error.toString()}`);
          }
          resolve(result);
        }
      );
    });
    let response = await req;
    // We are only interested in errors. Error on processing the request.
    if (response.hasOwnProperty("error")) {
      let e = JSON.parse(fs.readFileSync(ERROR_FILE));
      let current = e.errors;
      current.push({
        block_number: n,
        txn: txn,
        error: response.error,
      });
      // Update error file.
      fs.writeFileSync(ERROR_FILE, JSON.stringify(current));
    }
  }
  // Return the number of transactions processed in this block.
  return block.transactions.length;
}

(async () => {
  let web3 = new Web3(URL);
  // Check if there is connectivity.
  await web3.eth.net
    .isListening()
    .then(() => {})
    .catch((e) => {
      throw Error("Url cannot be accessed. Exit.");
    });

  // Create db directory if not exists.
  if (!fs.existsSync("db")) {
    fs.mkdirSync("db");
  }

  // Create error file if not exists.
  createErrorFile();

  // Get last processed block number. Create progress file if not exists.
  let last = lastProcessedBlock();
  let from = last.lastProcessedBlock;
  let totalTxn = last.ethTransactionsProcessed;
  let to = await web3.eth.getBlockNumber();

  // Progress is corrupted
  // a.k.a. network purged but progress file still holding previous progress.
  if (from >= to) {
    throw Error("Outdated progress file.");
  }

  let i;
  for (i = from + 1; i <= to; i++) {
    // Process a single block.
    totalTxn += await processBlock(web3, i);
    // Update progress.
    fs.writeFileSync(
      PROGRESS_FILE,
      JSON.stringify({
        lastProcessedBlock: i,
        ethTransactionsProcessed: totalTxn,
      })
    );
  }
})();
