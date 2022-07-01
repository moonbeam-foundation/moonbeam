import { AccessListish } from "@ethersproject/transactions";
import { ethers } from "ethers";
import fetch from "node-fetch";
import * as RLP from "rlp";
import { Contract } from "web3-eth-contract";

import { alith, ALITH_PRIVATE_KEY, baltathar, BALTATHAR_PRIVATE_KEY } from "./accounts";
import { getCompiled } from "./contracts";
import { customWeb3Request } from "./providers";
import { DevTestContext } from "./setup-dev-tests";

// Ethers is used to handle post-london transactions
import type { ApiPromise } from "@polkadot/api";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
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
  value?: string | number;
  data?: string;
  accessList?: AccessListish; // AccessList | Array<[string, Array<string>]>
}

export const TRANSACTION_TEMPLATE: TransactionOptions = {
  nonce: null,
  gas: 12_000_000,
  gasPrice: 1_000_000_000,
  value: "0x00",
};

export const ALITH_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: alith.address,
  privateKey: ALITH_PRIVATE_KEY,
};

export const BALTATHAR_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: baltathar.address,
  privateKey: BALTATHAR_PRIVATE_KEY,
};

export const createTransaction = async (
  context: DevTestContext,
  options: TransactionOptions
): Promise<string> => {
  const isLegacy = context.ethTransactionType === "Legacy";
  const isEip2930 = context.ethTransactionType === "EIP2930";
  const isEip1559 = context.ethTransactionType === "EIP1559";

  const gasPrice = options.gasPrice !== undefined ? options.gasPrice : 1_000_000_000;
  const maxPriorityFeePerGas =
    options.maxPriorityFeePerGas !== undefined ? options.maxPriorityFeePerGas : 0;
  const value = options.value !== undefined ? options.value : "0x00";
  const from = options.from || alith.address;
  const privateKey = options.privateKey !== undefined ? options.privateKey : ALITH_PRIVATE_KEY;

  // Instead of hardcoding the gas limit, we estimate the gas
  const gas =
    options.gas ||
    (await context.web3.eth.estimateGas({
      from: from,
      to: options.to,
      data: options.data,
    }));

  const maxFeePerGas = options.maxFeePerGas || 1_000_000_000;
  const accessList = options.accessList || [];
  const nonce = options.nonce || (await context.web3.eth.getTransactionCount(from, "pending"));

  let data, rawTransaction;
  if (isLegacy) {
    data = {
      from,
      to: options.to,
      value: value && value.toString(),
      gasPrice,
      gas,
      nonce: nonce,
      data: options.data,
    };
    const tx = await context.web3.eth.accounts.signTransaction(data, privateKey);
    rawTransaction = tx.rawTransaction;
  } else {
    const signer = new ethers.Wallet(privateKey, context.ethers);
    const chainId = await context.web3.eth.getChainId();
    if (isEip2930) {
      data = {
        from,
        to: options.to,
        value: value && value.toString(),
        gasPrice,
        gasLimit: gas,
        nonce: nonce,
        data: options.data,
        accessList,
        chainId,
        type: 1,
      };
    } else if (isEip1559) {
      data = {
        from,
        to: options.to,
        value: value && value.toString(),
        maxFeePerGas,
        maxPriorityFeePerGas,
        gasLimit: gas,
        nonce: nonce,
        data: options.data,
        accessList,
        chainId,
        type: 2,
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
      (data.gasPrice ? `gasPrice: ${data.gasPrice.toString()}, ` : "") +
      (data.maxFeePerGas ? `maxFeePerGas: ${data.maxFeePerGas.toString()}, ` : "") +
      (data.maxPriorityFeePerGas
        ? `maxPriorityFeePerGas: ${data.maxPriorityFeePerGas.toString()}, `
        : "") +
      (data.accessList ? `accessList: ${data.accessList.toString()}, ` : "") +
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
  options: TransactionOptions = ALITH_TRANSACTION_TEMPLATE
): Promise<string> => {
  return await createTransaction(context, {
    ...options,
    value: value.toString(),
    to,
  });
};

// Will create the transaction to deploy a contract.
// This requires to compute the nonce. It can't be used multiple times in the same block from the
// same from
export async function createContract(
  context: DevTestContext,
  contractName: string,
  options: TransactionOptions = ALITH_TRANSACTION_TEMPLATE,
  contractArguments: any[] = []
): Promise<{ rawTx: string; contract: Contract; contractAddress: string }> {
  const contractCompiled = getCompiled(contractName);
  const from = options.from !== undefined ? options.from : alith.address;
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
  options: TransactionOptions = ALITH_TRANSACTION_TEMPLATE
) {
  const rawTx = await createTransaction(context, {
    ...options,
    to: execution.contract.options.address,
    data: execution.contractCall.encodeABI(),
  });

  return rawTx;
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

  return context.createBlock(
    createTransaction(context, {
      from,
      privateKey,
      value: "0x0",
      gas: "0x200000",
      gasPrice: ALITH_TRANSACTION_TEMPLATE.gasPrice,
      to: precompileContractAddress,
      data,
    })
  );
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
      from: alith.address,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: precompileContractAddress,
      data,
    },
  ]);
}

export const sendAllStreamAndWaitLast = async (
  api: ApiPromise,
  extrinsics: SubmittableExtrinsic[],
  { threshold = 500, batch = 200, timeout = 120000 } = {
    threshold: 500,
    batch: 200,
    timeout: 120000,
  }
) => {
  let promises = [];
  while (extrinsics.length > 0) {
    const pending = await api.rpc.author.pendingExtrinsics();
    if (pending.length < threshold) {
      const chunk = extrinsics.splice(0, Math.min(threshold - pending.length, batch));
      // console.log(`Sending ${chunk.length}tx (${extrinsics.length} left)`);
      promises.push(
        Promise.all(
          chunk.map((tx) => {
            return new Promise(async (resolve, reject) => {
              let unsub: () => void;
              const timer = setTimeout(() => {
                reject(`timed out`);
                unsub();
              }, timeout);
              unsub = await tx.send((result) => {
                // reset the timer
                if (result.isError) {
                  console.log(result.toHuman());
                  clearTimeout(timer);
                  reject(result.toHuman());
                }
                if (result.isInBlock) {
                  unsub();
                  clearTimeout(timer);
                  resolve(null);
                }
              });
            }).catch((e) => {});
          })
        )
      );
    }
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }
  await Promise.all(promises);
};
