import Web3 from "web3";

import { JsonRpcResponse } from "web3-core-helpers";
import { SignedTransaction, TransactionConfig } from "web3-core";
import {
  basicTransfertx,
  CompleteTransactionConfig,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../constants";
import { wrappedCustomRequest } from "./web3Requests";
import { createAndFinalizeBlock } from ".";
import { Context, log } from "./testWithMoonbeam";

function isSignedTransaction(tx: Error | SignedTransaction): tx is SignedTransaction {
  return (tx as SignedTransaction).rawTransaction !== undefined;
}
function isJsonRpcResponse(res: Error | JsonRpcResponse): res is JsonRpcResponse {
  return (res as JsonRpcResponse).jsonrpc !== undefined;
}

// sign tx with error catching
async function wrappedSignTx(
  web3: Web3,
  txConfig: TransactionConfig,
  privateKey: string
): Promise<SignedTransaction | Error> {
  try {
    let tx = await web3.eth.accounts.signTransaction(txConfig, privateKey);
    return tx;
  } catch (e) {
    return new Error(e.toString());
  }
}

// Sign tx sequentially
async function serialSignTx(
  web3: Web3,
  n: number,
  startingNonce: number,
  customTxConfig: TransactionConfig
): Promise<(Error | SignedTransaction)[]> {
  const resArray = [];
  for (let index = 0; index < n; index++) {
    resArray.push(
      await wrappedSignTx(
        web3,
        { ...customTxConfig, nonce: startingNonce + index },
        GENESIS_ACCOUNT_PRIVATE_KEY
      )
    );
  }
  return resArray;
}

// Send tx to the pool sequentially
async function serialSendTx(
  web3: Web3,
  n: number,
  _txList: (Error | SignedTransaction)[]
): Promise<(Error | JsonRpcResponse)[]> {
  const resArray = [];
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

interface FillBlockReport {
  txPassed: number;
  txPassedFirstBlock: number;
  numberOfBlocks: number;
  signingTime: number;
  sendingTime: number;
}

export interface ErrorReport {
  [key: string]: {
    [key: string]: number;
  };
}

// This function sends a batch of signed transactions to the pool and records both
// how many tx were included in the first block and the total number of tx that were
// included in a block
// By default, the tx is a simple transfer, but a TransactionConfig can be specified as an option
export async function fillBlockWithTx(
  context: Context,
  numberOfTx: number,
  customTxConfig: CompleteTransactionConfig = basicTransfertx
): Promise<FillBlockReport> {
  let nonce: number = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);

  const numberArray = new Array(numberOfTx).fill(1);

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

  // First sign all transactions

  let txList: (Error | SignedTransaction)[] = await serialSignTx(
    context.web3,
    numberOfTx,
    nonce,
    customTxConfig as any // needed as the web3 types don't support chainId but the code does.
  );

  const signingTime: number = Date.now() - startSigningTime;

  log("Time it took to sign " + txList.length + " tx is " + signingTime / 1000 + " seconds");

  const startSendingTime: number = Date.now();

  //Then, send them to the pool

  let respList: (Error | JsonRpcResponse)[] = await serialSendTx(context.web3, numberOfTx, txList);

  respList.forEach((res) => {
    if (isJsonRpcResponse(res) && res.error) {
      reportError(res.error, "customreq");
    } else if (!isJsonRpcResponse(res)) {
      reportError(res, "signing");
    }
  });

  const sendingTime: number = Date.now() - startSendingTime;

  log("Time it took to send " + respList.length + " tx is " + sendingTime / 1000 + " seconds");

  log("Error Report : ", errorReport);

  log("created block in ", (await createAndFinalizeBlock(context.polkadotApi)) / 1000, " seconds");

  let numberOfBlocks = 0;
  let block = await context.web3.eth.getBlock("latest");
  let txPassed: number = block.transactions.length;
  const txPassedFirstBlock: number = txPassed;
  log(
    "block.gasUsed",
    block.gasUsed,
    "block.number",
    block.number,
    "block.transactions.length",
    block.transactions.length
  );

  let i: number = 2;

  while (block.transactions.length !== 0) {
    await createAndFinalizeBlock(context.polkadotApi);

    block = await context.web3.eth.getBlock("latest");
    log(
      "following block, block" + i + ".gasUsed",
      block.gasUsed,
      "block" + i + ".number",
      block.number,
      "block" + i + ".transactions.length",
      block.transactions.length
    );
    txPassed += block.transactions.length;
    numberOfTx += 1;
    i += 1;
  }

  return { txPassed, txPassedFirstBlock, sendingTime, signingTime, numberOfBlocks };
}
