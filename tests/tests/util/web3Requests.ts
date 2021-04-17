import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";

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
