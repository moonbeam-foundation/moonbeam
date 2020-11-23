import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";

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
            //`Failed to send custom request (${method} (${params.join(",")})): ${
            error.message || error.toString()
            //}`
          );
        }
        resolve(result);
      }
    );
  });
}

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
export async function createAndFinalizeBlock(web3: Web3) {
  const response: JsonRpcResponse = await customRequest(web3, "engine_createBlock", [
    true,
    true,
    null,
  ]);
  if (response.error) {
    console.log("error during block creation");
    throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
  }
}
