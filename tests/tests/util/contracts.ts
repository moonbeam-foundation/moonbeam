import { ApiPromise } from "@polkadot/api";
import { TransactionReceipt } from "web3-core";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth-contract";
import { createAndFinalizeBlock } from "./polkadotApiRequests";
import { Context } from "./testWithMoonbeam";
import Web3 from "web3";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../constants";
import { customRequest } from ".";
import { contractSources } from "../contracts/sources";

export interface Compiled {
  byteCode: string;
  contract: any;
  sourceCode: string;
}

const contracts: { [name: string]: Compiled } = {};
const contractObs: { [name: string]: ((Compiled) => void)[] } = {};
export async function getCompiled(name: string): Promise<Compiled> {
  if (!contractSources[name]) {
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);
  }
  if (contracts[name]) {
    return contracts[name];
  }
  const promise = new Promise<Compiled>((resolve) => {
    const shouldLoad = !contractObs[name];
    if (!contractObs[name]) {
      contractObs[name] = [];
    }
    contractObs[name].push(resolve);
    if (shouldLoad) {
      // Will load the contract async and callback all the promise waiting for this contract.
      setImmediate(() => {
        try {
          console.log(`Loading ${name}.json`);
          contracts[name] = require(`../../contracts/compiled/${name}.json`);
        } catch (e) {
          throw new Error(
            `Contract name ${name} is not compiled. (should be done in mochaGlobalSetup)`
          );
        }

        // Call back all the pending promises and clear the list.
        contractObs[name].forEach((resolvePending) => {
          resolvePending(contracts[name]);
        });
        delete contractObs[name];
      });
    }
  });
  return promise;
}

export async function deployContractByName(
  api: ApiPromise,
  web3: Web3,
  name: string
): Promise<Contract> {
  const contractCompiled = await getCompiled(name);
  return deployContractManualSeal(
    api,
    web3,
    contractCompiled.byteCode,
    contractCompiled.contract.abi
  );
}

// Deploy and instantiate a contract with manuel seal
export async function deployContractManualSeal(
  api: ApiPromise,
  web3: Web3,
  contractByteCode: string,
  contractABI: AbiItem[],
  account: string = GENESIS_ACCOUNT,
  privateKey: string = GENESIS_ACCOUNT_PRIVATE_KEY
): Promise<Contract> {
  const tx = await web3.eth.accounts.signTransaction(
    {
      from: account,
      data: contractByteCode,
      value: "0x00",
      gasPrice: "0x01",
      gas: "0x100000",
    },
    privateKey
  );
  await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);
  await createAndFinalizeBlock(api);
  let rcpt: TransactionReceipt = await web3.eth.getTransactionReceipt(tx.transactionHash);
  return new web3.eth.Contract(contractABI, rcpt.contractAddress);
}

interface FnCallOptions {
  account?: string;
  privateKey?: string;
  gas?: string;
}

// Call a function from a contract instance using manual seal
export async function callContractFunctionMS(
  context: Context,
  contractAddress: string,
  bytesCode: string,
  options?: FnCallOptions
) {
  try {
    const contractCall = {
      from: options && options.account ? options.account : GENESIS_ACCOUNT,
      to: contractAddress,
      data: bytesCode,
      gasPrice: "0x01",
      gas: options && options.gas ? options.gas : "0x100000",
    };
    const txCall = await context.web3.eth.accounts.signTransaction(
      contractCall,
      options && options.privateKey ? options.privateKey : GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [txCall.rawTransaction]);
    return await createAndFinalizeBlock(context.polkadotApi);
  } catch (e) {
    console.log("error caught during callContractFunctionMS", e);
    throw new Error(e);
  }
}
