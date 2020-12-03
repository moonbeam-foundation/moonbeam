import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { TransactionReceipt } from "web3-core";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth-contract";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../constants";

// make a web3 request, adapted to manual seal testing
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

// wrap the above function to catch errors and return them into a JsonRpc format
export async function wrappedCustomRequest(
  web3: Web3,
  method: string,
  params: any[]
): Promise<JsonRpcResponse> {
  try {
    let resp = await customRequest(web3, method, params);
    return resp;
  } catch (e) {
    return {
      jsonrpc: "req error",
      id: 0,
      error: typeof e === "string" ? e.toString() : JSON.stringify(e),
    };
  }
}

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(web3: Web3): Promise<number> {
  const startTime: number = Date.now();
  try {
    const response: JsonRpcResponse = await customRequest(web3, "engine_createBlock", [
      true,
      true,
      null,
    ]);
    if (response.error) {
      console.log("error during block creation");
      throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
    }
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }
  return Date.now() - startTime;
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
      gasPrice: "0x01",
      gas: "0x100000",
    },
    privateKey
  );
  await customRequest(web3, "eth_sendRawTransaction", [tx.rawTransaction]);
  await createAndFinalizeBlock(web3);
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
  web3: Web3,
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
    const txCall = await web3.eth.accounts.signTransaction(
      contractCall,
      options && options.privateKey ? options.privateKey : GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(web3, "eth_sendRawTransaction", [txCall.rawTransaction])
    return await createAndFinalizeBlock(web3);
  } catch (e) {
    console.log("error caught during callContractFunctionMS", e);
    throw new Error(e);
  }
}
