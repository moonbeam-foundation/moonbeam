import { customWeb3Request } from "./providers";
import { TransactionReceipt } from "web3-core";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth-contract";
import Web3 from "web3";

import { contractSources } from "../contracts/sources";
import { alith, ALITH_PRIVATE_KEY } from "./accounts";

export interface Compiled {
  byteCode: string;
  contract: any;
  sourceCode: string;
}

const contracts: { [name: string]: Compiled } = {};
export function getCompiled(name: string): Compiled {
  if (!contractSources[name]) {
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);
  }
  if (!contracts[name]) {
    try {
      contracts[name] = require(`../contracts/compiled/${name}.json`);
    } catch (e) {
      throw new Error(
        `Contract name ${name} is not compiled. Please run 'npm run pre-build-contracts`
      );
    }
  }

  return contracts[name];
}

// Deploy and instantiate a contract with manuel seal
export async function deployContractManualSeal(
  web3: Web3,
  contractByteCode: string,
  contractABI: AbiItem[],
  account: string = alith.address,
  privateKey: string = ALITH_PRIVATE_KEY
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
