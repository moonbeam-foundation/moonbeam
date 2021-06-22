import { customWeb3Request } from "./providers";
import { TransactionReceipt } from "web3-core";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth-contract";
import Web3 from "web3";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";
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
          contracts[name] = require(`../contracts/compiled/${name}.json`);
        } catch (e) {
          throw new Error(
            `Contract name ${name} is not compiled. Please run 'npm run pre-build-contracts`
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

// Deploy and instantiate a contract with manuel seal
export async function deployContractManualSeal(
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
      gasPrice: 1_000_000_000,
      gas: "0x100000",
    },
    privateKey
  );
  await customWeb3Request(web3, "eth_sendRawTransaction", [tx.rawTransaction]);
  let rcpt: TransactionReceipt = await web3.eth.getTransactionReceipt(tx.transactionHash);
  return new web3.eth.Contract(contractABI, rcpt.contractAddress);
}
