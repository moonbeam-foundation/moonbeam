import Web3 from "web3";
import { ApiPromise, WsProvider } from "@polkadot/api";

import { JsonRpcResponse } from "web3-core-helpers";
import { spawn, ChildProcess } from "child_process";
import {SignedTransaction, TransactionConfig} from 'web3-core'
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

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(web3: Web3) {
  const response = await customRequest(web3, "engine_createBlock", [true, true, null]);
  if (!response.result) {
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

export async function fillBlockWithTx(context: { web3: Web3 },numberOfTx:number, expectFunction){
  let nonce:number= await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)

  const numberArray=new Array(numberOfTx).fill(1)

  interface ErrorReport {
    [key:string]:{
      [key:string]:number
    }
  }

  let errorReport:ErrorReport={
    signing:{},
    customreq:{}
  }
  function reportError(e,context:string){
    let message:string=e.error?e.error.message:e
    if (errorReport[context][message]){
      errorReport[context][message]+=1
    }else {
      errorReport[context][message]=1
    }
  }
    
  await Promise.all(numberArray.map(async(_,i)=>{
    let tx:SignedTransaction;
    // sign tx
    try{
      tx = await context.web3.eth.accounts.signTransaction(
        {...basicTransfertx,nonce:nonce+i},
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
    }catch(e){
      reportError(e,'signing')
    }
    // send it
    try{
      return customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    }catch(e){
      reportError(e,'customreq')
    }
  })) 
  
  console.log('Error Report : ',errorReport)

  await createAndFinalizeBlock(context.web3);
  const block = await context.web3.eth.getBlock("latest");
  console.log('block.gasUsed',block.gasUsed,'block.number',block.number,'block.transactions.length',block.transactions.length)
  expectFunction(block.transactions.length).to.eq(numberOfTx)
}
