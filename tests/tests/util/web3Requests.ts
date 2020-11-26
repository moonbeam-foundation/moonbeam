import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { TransactionReceipt,  } from "web3-core"; 
import { AbiItem } from "web3-utils";
import {Contract} from 'web3-eth-contract';

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
    console.log("thrown error in wrapped custom req");
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
  } catch(e){
      console.log('ERROR DURING BLOCK FINALIZATION',e)
  }
  return Date.now() - startTime;
}

// Deploy and instantiate a contract with manuel seal
export async function deployContractManualSeal(web3:Web3, contractByteCode:string,contractABI:AbiItem[],account:string,privateKey:string):Promise<Contract>{
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
