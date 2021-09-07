import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";
import Web3 from "web3";
import * as RLP from "rlp";
import { getCompiled } from "./contracts";
import { Contract } from "web3-eth-contract";
import { DevTestContext } from "./setup-dev-tests";
import { customWeb3Request } from "./providers";
const debug = require("debug")("test:transaction");

export interface TransactionOptions {
  from?: string;
  to?: string;
  privateKey?: string;
  nonce?: number;
  gas?: string | number;
  gasPrice?: string | number;
  value?: string | number | BigInt;
  data?: string;
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
  web3: Web3,
  options: TransactionOptions
): Promise<string> => {
  const gas = options.gas || 12_000_000;
  const gasPrice = options.gasPrice !== undefined ? options.gasPrice : 1_000_000_000;
  const value = options.value !== undefined ? options.value : "0x00";
  const from = options.from || GENESIS_ACCOUNT;
  const privateKey =
    options.privateKey !== undefined ? options.privateKey : GENESIS_ACCOUNT_PRIVATE_KEY;

  const data = {
    from,
    to: options.to,
    value: value && value.toString(),
    gasPrice,
    gas,
    nonce: options.nonce,
    data: options.data,
  };
  debug(
    `Tx [${/:([0-9]+)$/.exec((web3.currentProvider as any).host)[1]}] ` +
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
  const tx = await web3.eth.accounts.signTransaction(data, privateKey);
  return tx.rawTransaction;
};

export const createTransfer = async (
  web3: Web3,
  to: string,
  value: number | string | BigInt,
  options: TransactionOptions = GENESIS_TRANSACTION
): Promise<string> => {
  return await createTransaction(web3, { ...options, value, to });
};

// Will create the transaction to deploy a contract.
// This requires to compute the nonce. It can't be used multiple times in the same block from the
// same from
export async function createContract(
  web3: Web3,
  contractName: string,
  options: TransactionOptions = GENESIS_TRANSACTION,
  contractArguments: any[] = []
): Promise<{ rawTx: string; contract: Contract; contractAddress: string }> {
  const contractCompiled = await getCompiled(contractName);
  const from = options.from !== undefined ? options.from : GENESIS_ACCOUNT;
  const nonce = options.nonce || (await web3.eth.getTransactionCount(from));
  const contractAddress =
    "0x" +
    web3.utils
      .sha3(RLP.encode([from, nonce]) as any)
      .slice(12)
      .substring(14);

  const contract = new web3.eth.Contract(contractCompiled.contract.abi, contractAddress);
  const data = contract
    .deploy({
      data: contractCompiled.byteCode,
      arguments: contractArguments,
    })
    .encodeABI();

  const rawTx = await createTransaction(web3, { ...options, from, nonce, data });

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
  web3: Web3,
  execution: {
    contract: Contract;
    contractCall: any;
  },
  options: TransactionOptions = GENESIS_TRANSACTION
) {
  const tx = await createTransaction(web3, {
    ...options,
    to: execution.contract.options.address,
    data: execution.contractCall.encodeABI(),
  });

  return tx;
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

  const tx = await createTransaction(context.web3, {
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
