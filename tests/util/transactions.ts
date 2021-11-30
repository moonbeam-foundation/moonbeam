import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";
import Web3 from "web3";
import * as RLP from "rlp";
import { getCompiled } from "./contracts";
import { Contract } from "web3-eth-contract";
import fetch from "node-fetch";
import { DevTestContext } from "./setup-dev-tests";
import { customWeb3Request } from "./providers";
// Ethers is used to handle post-london transactions
import { ethers } from "ethers";
import { AccessListish } from "@ethersproject/transactions";
const debug = require("debug")("test:transaction");

export interface TransactionOptions {
  from?: string;
  to?: string;
  privateKey?: string;
  nonce?: number;
  gas?: string | number;
  gasPrice?: string | number;
  maxFeePerGas?: string | number;
  maxPriorityFeePerGas?: string | number;
  value?: string | number | BigInt;
  data?: string;
  accessList?: AccessListish; // AccessList | Array<[string, Array<string>]>
}

export const GENESIS_TRANSACTION: TransactionOptions = {
  from: GENESIS_ACCOUNT,
  privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
  nonce: null,
  gas: 12_000_000,
  gasPrice: 1_000_000_000,
  value: "0x00",
};

export const createTransaction = async (
  context: DevTestContext,
  options: TransactionOptions
): Promise<string> => {

  const isLegacy = options.maxFeePerGas === undefined &&
    options.maxPriorityFeePerGas === undefined && options.accessList === undefined;
  const isEip2930 = options.maxFeePerGas === undefined && options.accessList !== undefined;
  const isEip1559 = options.maxFeePerGas !== undefined;

  const gas = options.gas || 12_000_000;
  const gasPrice = options.gasPrice !== undefined ? options.gasPrice : 1_000_000_000;
  const maxPriorityFeePerGas = options.maxPriorityFeePerGas !== undefined ?
    options.maxPriorityFeePerGas : 0;
  const value = options.value !== undefined ? options.value : "0x00";
  const from = options.from || GENESIS_ACCOUNT;
  const privateKey =
    options.privateKey !== undefined ? options.privateKey : GENESIS_ACCOUNT_PRIVATE_KEY;

  let data, rawTransaction;
  if(isLegacy) {
    data = {
      from,
      to: options.to,
      value: value && value.toString(),
      gasPrice,
      gas,
      nonce: options.nonce,
      data: options.data,
    };
    const tx = await context.web3.eth.accounts.signTransaction(data, privateKey);
    rawTransaction = tx.rawTransaction;
  } else {
    const signer = new ethers.Wallet(privateKey, context.ethers);
    if(isEip2930) {
      data = {
        from,
        to: options.to,
        value: value && value.toString(),
        gasPrice,
        gasLimit: gas,
        nonce: options.nonce,
        data: options.data,
        accessList: options.accessList,
      };
    } else if(isEip1559) {
      data = {
        from,
        to: options.to,
        value: value && value.toString(),
        maxFeePerGas: options.maxFeePerGas,
        maxPriorityFeePerGas,
        gasLimit: gas,
        nonce: options.nonce,
        data: options.data,
        accessList: options.accessList,
      };
    }
    rawTransaction = await signer.signTransaction(data);
  }

  debug(
    `Tx [${/:([0-9]+)$/.exec((context.web3.currentProvider as any).host)[1]}] ` +
      `from: ${data.from.substr(0, 5) + "..." + data.from.substr(data.from.length - 3)}, ` +
      (data.to
        ? `to: ${data.to.substr(0, 5) + "..." + data.to.substr(data.to.length - 3)}, `
        : "") +
      (data.value ? `value: ${data.value.toString()}, ` : "") +
      `gasPrice: ${data.gasPrice.toString()}, ` +
      (data.gas ? `gas: ${data.gas.toString()}, ` : "") +
      (data.nonce ? `nonce: ${data.nonce.toString()}, ` : "") +
      (!data.data
        ? ""
        : `data: ${
            data.data.length < 50
              ? data.data
              : data.data.substr(0, 5) + "..." + data.data.substr(data.data.length - 3)
          }`)
  );
  return rawTransaction;
};

export const createTransfer = async (
  context: DevTestContext,
  to: string,
  value: number | string | BigInt,
  options: TransactionOptions = GENESIS_TRANSACTION
): Promise<string> => {
  return await createTransaction(context, { ...options, value, to });
};

// Will create the transaction to deploy a contract.
// This requires to compute the nonce. It can't be used multiple times in the same block from the
// same from
export async function createContract(
  context: DevTestContext,
  contractName: string,
  options: TransactionOptions = GENESIS_TRANSACTION,
  contractArguments: any[] = []
): Promise<{ rawTx: string; contract: Contract; contractAddress: string }> {
  const contractCompiled = await getCompiled(contractName);
  const from = options.from !== undefined ? options.from : GENESIS_ACCOUNT;
  const nonce = options.nonce || (await context.web3.eth.getTransactionCount(from));
  const contractAddress =
    "0x" +
    context.web3.utils
      .sha3(RLP.encode([from, nonce]) as any)
      .slice(12)
      .substring(14);

  const contract = new context.web3.eth.Contract(contractCompiled.contract.abi, contractAddress);
  const data = contract
    .deploy({
      data: contractCompiled.byteCode,
      arguments: contractArguments,
    })
    .encodeABI();

  const rawTx = await createTransaction(context, { ...options, from, nonce, data });

  return {
    rawTx,
    contract,
    contractAddress,
  };
}

// Will create the transaction to execute a contract function.
// This requires to compute the nonce. It can't be used multiple times in the same block from the
// same from
export async function createContractExecution(
  context: DevTestContext,
  execution: {
    contract: Contract;
    contractCall: any;
  },
  options: TransactionOptions = GENESIS_TRANSACTION
) {
  const tx = await createTransaction(context, {
    ...options,
    to: execution.contract.options.address,
    data: execution.contractCall.encodeABI(),
  });

  return tx;
}

/**
 * Send a JSONRPC request to the node at http://localhost:9933.
 *
 * @param method - The JSONRPC request method.
 * @param params - The JSONRPC request params.
 */
export function rpcToLocalNode(rpcPort: number, method: string, params: any[] = []): Promise<any> {
  return fetch("http://localhost:" + rpcPort, {
    body: JSON.stringify({
      id: 1,
      jsonrpc: "2.0",
      method,
      params,
    }),
    headers: {
      "Content-Type": "application/json",
    },
    method: "POST",
  })
    .then((response) => response.json())
    .then(({ error, result }) => {
      if (error) {
        throw new Error(`${error.code} ${error.message}: ${JSON.stringify(error.data)}`);
      }

      return result;
    });
}
// The parameters passed to the function are assumed to have all been converted to hexadecimal
export async function sendPrecompileTx(
  context: DevTestContext,
  precompileContractAddress: string,
  selectors: { [key: string]: string },
  from: string,
  privateKey: string,
  selector: string,
  parameters: string[]
) {
  let data: string;
  if (selectors[selector]) {
    data = `0x${selectors[selector]}`;
  } else {
    throw new Error(`selector doesn't exist on the precompile contract`);
  }
  parameters.forEach((para: string) => {
    data += para.slice(2).padStart(64, "0");
  });

  const tx = await createTransaction(context, {
    from,
    privateKey,
    value: "0x0",
    gas: "0x200000",
    gasPrice: GENESIS_TRANSACTION.gasPrice,
    to: precompileContractAddress,
    data,
  });

  return context.createBlock({
    transactions: [tx],
  });
}

const GAS_PRICE = "0x" + (1_000_000_000).toString(16);
export async function callPrecompile(
  context: DevTestContext,
  precompileContractAddress: string,
  selectors: { [key: string]: string },
  selector: string,
  parameters: string[]
) {
  let data: string;
  if (selectors[selector]) {
    data = `0x${selectors[selector]}`;
  } else {
    throw new Error(`selector doesn't exist on the precompile contract`);
  }
  parameters.forEach((para: string) => {
    data += para.slice(2).padStart(64, "0");
  });

  return await customWeb3Request(context.web3, "eth_call", [
    {
      from: GENESIS_ACCOUNT,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: precompileContractAddress,
      data,
    },
  ]);
}
