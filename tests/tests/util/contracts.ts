import { ApiPromise } from "@polkadot/api";
import { TransactionReceipt } from "web3-core";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth-contract";
import { createAndFinalizeBlock } from "./polkadotApiRequests";
import { Context } from "./testWithMoonbeam";
import solc from "solc";
import Web3 from "web3";
import fs from "fs";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../constants";
import { customRequest } from ".";
import { contractSources } from "../constants/contractSources";

export function compileSolidity(contractContent: string, contractName: string = "Test"): Compiled {
  let result = JSON.parse(
    solc.compile(
      JSON.stringify({
        language: "Solidity",
        sources: {
          "main.sol": {
            content: contractContent,
          },
        },
        settings: {
          outputSelection: {
            "*": {
              "*": ["*"],
            },
          },
        },
      })
    )
  );

  const contract = result.contracts["main.sol"][contractName];
  return {
    byteCode: "0x" + contract.evm.bytecode.object,
    contract,
    sourceCode: contractContent,
  };
}
export interface Compiled {
  byteCode: string;
  contract: any;
  sourceCode: string;
}

export async function getCompiled(name: string): Promise<Compiled> {
  if (!contractSources[name])
    throw new Error(`Contract name (${name}) doesn't exist in test suite`);
  let finalCompiled: Compiled = await new Promise<Compiled>((res) => {
    fs.readFile(`./tests/constants/compiledContracts/${name}.json`, async (err, data) => {
      if (err) {
        const contractCompiled = compileSolidity(contractSources[name], name);
        let compiled = JSON.stringify(contractCompiled);
        await new Promise<void>((res2) => {
          fs.writeFile(
            `./tests/constants/compiledContracts/${name}.json`,
            compiled,
            {
              flag: "w",
            },
            (err) => {
              if (err) {
                console.log("error whilst writing,e", err);
              }
              console.log("New compiled contract file has been saved!");
              res2();
            }
          );
        });
        res(contractCompiled);
      } else {
        res(JSON.parse(data.toString()));
      }
    });
  });
  return finalCompiled;
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
