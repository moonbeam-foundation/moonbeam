import Web3 from "web3";
import { ApiPromise, WsProvider } from "@polkadot/api";

import { JsonRpcResponse } from "web3-core-helpers";
import { spawn, ChildProcess } from "child_process";
import { SignedTransaction, TransactionConfig } from "web3-core";
import { basicTransfertx, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

export const PORT = 19931;
export const RPC_PORT = 19932;
export const WS_PORT = 19933;
export const SPECS_PATH = `./moonbeam-test-specs`;

export const DISPLAY_LOG = process.env.MOONBEAM_LOG || false;
export const MOONBEAM_LOG = process.env.MOONBEAM_LOG || "info";

export const BINARY_PATH =
  process.env.BINARY_PATH || `../node/standalone/target/release/moonbase-standalone`;
export const SPAWNING_TIME = 30000;

function isSignedTransaction(tx: Error | SignedTransaction): tx is SignedTransaction {
  return (tx as SignedTransaction).rawTransaction !== undefined;
}
function isJsonRpcResponse(res: Error | JsonRpcResponse): res is JsonRpcResponse {
  return (res as JsonRpcResponse).jsonrpc !== undefined;
}

export async function customRequest(web3: Web3, method: string, params: any[]) {
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
            //`Failed to send custom request (${method} (${params.join(",")})): ${
            error.message || error.toString()
            //}`
          );
        }
        resolve(result);
      }
    );
  });
}
export async function wrappedCustomRequest(
  web3: Web3,
  method: string,
  params: any[]
): Promise<JsonRpcResponse> {
  try {
    let resp = await customRequest(web3, method, params);
    return resp;
  } catch (e) {
    console.log("thrown error in wrapped custom req");
    return {
      jsonrpc: "req error",
      id: 0,
      error: typeof e === "string" ? e.toString() : JSON.stringify(e),
    };
  }
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(web3: Web3) {
  const response: JsonRpcResponse = await customRequest(web3, "engine_createBlock", [
    true,
    true,
    null,
  ]);
  if (response.error) {
    console.log("error during block creation");
    throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
  }
}

export interface Context {
  web3: Web3;

  // WsProvider for the PolkadotJs API
  wsProvider: WsProvider;
  polkadotApi: ApiPromise;
}

export async function startMoonbeamNode(
  specFilename: string,
  provider?: string
): Promise<{ context: Context; binary: ChildProcess }> {
  let web3;
  if (!provider || provider == "http") {
    web3 = new Web3(`http://localhost:${RPC_PORT}`);
  }

  const cmd = BINARY_PATH;
  const args = [
    `--chain=${SPECS_PATH}/${specFilename}`,
    `--validator`, // Required by manual sealing to author the blocks
    `--execution=Native`, // Faster execution using native
    `--no-telemetry`,
    `--no-prometheus`,
    `--manual-seal`,
    `--no-grandpa`,
    `--force-authoring`,
    `-l${MOONBEAM_LOG}`,
    `--port=${PORT}`,
    `--rpc-port=${RPC_PORT}`,
    `--ws-port=${WS_PORT}`,
    `--tmp`,
  ];
  const binary = spawn(cmd, args);
  binary.on("error", (err) => {
    if ((err as any).errno == "ENOENT") {
      console.error(
        `\x1b[31mMissing Moonbeam binary ` +
          `(${BINARY_PATH}).\nPlease compile the Moonbeam project\x1b[0m`
      );
    } else {
      console.error(err);
    }
    process.exit(1);
  });

  const binaryLogs = [];
  await new Promise((resolve) => {
    const timer = setTimeout(() => {
      console.error(`\x1b[31m Failed to start Moonbeam Test Node.\x1b[0m`);
      console.error(`Command: ${cmd} ${args.join(" ")}`);
      console.error(`Logs:`);
      console.error(binaryLogs.map((chunk) => chunk.toString()).join("\n"));
      process.exit(1);
    }, SPAWNING_TIME - 2000);

    const onData = async (chunk) => {
      if (DISPLAY_LOG) {
        console.log(chunk.toString());
      }
      binaryLogs.push(chunk);
      if (chunk.toString().match(/Manual Seal Ready/)) {
        if (!provider || provider == "http") {
          // This is needed as the EVM runtime needs to warmup with a first call
          await web3.eth.getChainId();
        }

        clearTimeout(timer);
        if (!DISPLAY_LOG) {
          binary.stderr.off("data", onData);
          binary.stdout.off("data", onData);
        }
        // console.log(`\x1b[31m Starting RPC\x1b[0m`);
        resolve();
      }
    };
    binary.stderr.on("data", onData);
    binary.stdout.on("data", onData);
  });

  const wsProvider = new WsProvider(`ws://localhost:${WS_PORT}`);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    types: {
      AccountId: "EthereumAccountId",
      Address: "AccountId",
      Balance: "u128",
      RefCount: "u8",
      // mapping the lookup
      LookupSource: "AccountId",
      Account: {
        nonce: "U256",
        balance: "u128",
      },
      Transaction: {
        nonce: "U256",
        action: "String",
        gas_price: "u64",
        gas_limit: "u64",
        value: "U256",
        input: "Vec<u8>",
        signature: "Signature",
      },
      Signature: {
        v: "u64",
        r: "H256",
        s: "H256",
      },
    },
  });

  if (provider == "ws") {
    web3 = new Web3(`ws://localhost:${WS_PORT}`);
  }

  return { context: { web3, polkadotApi, wsProvider }, binary };
}

export function describeWithMoonbeam(
  title: string,
  specFilename: string,
  cb: (context: Context) => void,
  provider?: string
) {
  describe(title, () => {
    let context: Context = { web3: null, wsProvider: null, polkadotApi: null };
    let binary: ChildProcess;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      this.timeout(SPAWNING_TIME);
      const init = await startMoonbeamNode(specFilename, provider);
      // Context is given prior to this assignement, so doing
      // context = init.context will fail because it replace the variable;
      context.web3 = init.context.web3;
      context.wsProvider = init.context.wsProvider;
      context.polkadotApi = init.context.polkadotApi;
      binary = init.binary;
    });

    after(async function () {
      // console.log(`\x1b[31m Killing RPC\x1b[0m`);
      context.wsProvider.disconnect();
      binary.kill();
      binary = null;
    });

    cb(context);
  });
}

//TODO: add description and specify test
// expectations should be separated from fun and ddisplayed in test file

export async function fillBlockWithTx(
  context: { web3: Web3 },
  numberOfTx: number,
  customTxConfig: TransactionConfig = basicTransfertx
) {
  let nonce: number = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);

  const numberArray = new Array(numberOfTx).fill(1);

  interface ErrorReport {
    [key: string]: {
      [key: string]: number;
    };
  }

  let errorReport: ErrorReport = {
    signing: {},
    customreq: {},
  };

  function reportError(e, domain: string) {
    let message: string = e.error ? e.error.message : e.message ? e.message : JSON.stringify(e);
    if (errorReport[domain][message]) {
      errorReport[domain][message] += 1;
    } else {
      errorReport[domain][message] = 1;
    }
  }

  const startSigningTime: number = Date.now();

  async function wrappedSignTx(
    web3:Web3,
    txConfig: TransactionConfig,
    privateKey: string
  ): Promise<SignedTransaction | Error> {
    try {
      let tx = await web3.eth.accounts.signTransaction(txConfig, privateKey);
      return tx;
    } catch (e) {
      //reportError(e, "signing");
      return new Error(e.toString());
    }
  }

  // First sign all transactions
  // let txList: (Error | SignedTransaction)[] = await Promise.all(
  //   numberArray.map(async (_, i) => {
  //     // sign tx
  //     return wrappedSignTx({ ...customTxConfig, nonce: nonce + i }, GENESIS_ACCOUNT_PRIVATE_KEY);
  //   })
  // );

    // async function serialSignTx(web3:Web3, n: number): Promise<(Error | SignedTransaction)[]> {
    //   if (n === 0) {
    //     return [];
    //   } else {
    //     const resArray: (Error | SignedTransaction)[] = await serialSignTx(web3,n - 1);
    //     console.log('i',nonce+n)
    //     resArray.push(
    //       await wrappedSignTx(web3,{ ...customTxConfig, nonce: nonce + n }, GENESIS_ACCOUNT_PRIVATE_KEY)
    //     );
    //     return resArray;
    //   }
    // }
  async function serialSignTx(web3:Web3, n: number, startingNonce:number): Promise<(Error | SignedTransaction)[]> {
    const resArray=[]
    for (let index = 0; index < n; index++) {
      resArray.push(
        await wrappedSignTx(web3,{ ...customTxConfig, nonce: startingNonce + index + 1 }, GENESIS_ACCOUNT_PRIVATE_KEY)
      );
    }
    return resArray;
  }

  let txList: (Error | SignedTransaction)[] = await serialSignTx(context.web3,numberOfTx,nonce);
  console.log('txList',txList)

  const endSigningTime: number = Date.now();

  console.log(
    "Time it took to sign " +
      txList.length +
      " tx is " +
      (endSigningTime - startSigningTime) / 1000 +
      " seconds"
  );

  const startSendingTime: number = Date.now();

  //Then, send them to the pool
  // let respList: JsonRpcResponse[] = await Promise.all(
  //   txList.map(async (tx, i) => {
  //     // send it
  //     if (isSignedTransaction(tx)) {
  //       return wrappedCustomRequest(context.web3, "eth_sendRawTransaction", [
  //         (tx as SignedTransaction).rawTransaction,
  //       ]);
  //     } else {
  //       return {
  //         jsonrpc: "signature error",
  //         id: 0,
  //         error: tx.message,
  //       };
  //     }
  //   })
  // );

  // async function serialSendTx(web3:Web3, n: number): Promise<(Error | JsonRpcResponse)[]> {
  //   if (n === 0) {
  //     return [];
  //   } else {
  //     const resArray: (Error | JsonRpcResponse)[] = await serialSendTx(web3,n - 1);
  //     if (isSignedTransaction(txList[n-1])) {
  //       resArray.push(
  //         await wrappedCustomRequest(web3, "eth_sendRawTransaction", [
  //           (txList[n-1] as SignedTransaction).rawTransaction,
  //         ])
  //       );
  //     } else {
  //       resArray.push(txList[n-1] as Error);
  //     }
  //     return resArray;
  //   }
  // }

  async function serialSendTx(web3:Web3, n: number, _txList: (Error | SignedTransaction)[]): Promise<(Error | JsonRpcResponse)[]> {
    const resArray=[]
    for (let index = 0; index < n; index++) {
      if (isSignedTransaction(_txList[index])) {
      resArray.push(
        await wrappedCustomRequest(web3, "eth_sendRawTransaction", [
                    (_txList[index] as SignedTransaction).rawTransaction,
                  ])
      );
                } else {
                       resArray.push(_txList[index] as Error);
                }
    }
    return resArray;
  }

  let respList: (Error | JsonRpcResponse)[] = await serialSendTx(context.web3,numberOfTx, txList);
  console.log('respList',respList)

  respList.forEach((res) => {
    //console.log('res',res)
    if (isJsonRpcResponse(res) && res.error) {
      //console.log("error in final array", res);
      //@ts-ignore
      reportError(res.error, "customreq"); //res.jsonrpc == "signature error" ? "signing" : "customreq");
    } else if (!isJsonRpcResponse(res)) {
      reportError(res, "signing");
    }
  });

  const endSendingTime: number = Date.now();

  console.log(
    "Time it took to send " +
      respList.length +
      " tx is " +
      (endSendingTime - startSendingTime) / 1000 +
      " seconds"
  );

  // TODO : use tx receipt to fetch tx status agao

  // create another blockc and see remaining tx

  // separate tx signing and tx sending

  // TODO : verify resp

  console.log("Error Report : ", errorReport);

  await createAndFinalizeBlock(context.web3);

  let block = await context.web3.eth.getBlock("latest");
  console.log(block)
  let txPassed: number = block.transactions.length;
  console.log(
    "block.gasUsed",
    block.gasUsed,
    "block.number",
    block.number,
    "block.transactions.length",
    block.transactions.length
  );

  let i: number = 2;

  while (i < 5) {
    //(block.transactions.length!==0){

    await createAndFinalizeBlock(context.web3);

    block = await context.web3.eth.getBlock("latest");
    console.log(
      "following block, block" + i + ".gasUsed",
      block.gasUsed,
      "block" + i + ".number",
      block.number,
      "block" + i + ".transactions.length",
      block.transactions.length
    );
    i += 1;
  }

  // await Promise.all(
  //   txList.map(async (tx, i) => {
  //     // send it
  //     if (isSignedTransaction(tx)) {
  //       try {
  //         console.log((tx as SignedTransaction).transactionHash); //return console.log(await context.web3.eth.getTransactionReceipt((tx as SignedTransaction).transactionHash))
  //       } catch (e) {
  //         console.log("gettxrceipt error", e);
  //       }
  //     }
  //   })
  // );

  return txPassed;
}

//todo: test web3 limits and serial vs parallel
