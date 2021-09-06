import Web3 from "web3";

import { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { JsonRpcResponse } from "web3-core-helpers";

export const sendAllAndWaitLast = async (extrinsics: SubmittableExtrinsic[]) => {
  return new Promise(async (resolve, reject) => {
    console.log(`Preparing to send ${extrinsics.length} extrinsics`);
    for (let i = 0; i < extrinsics.length; i++) {
      if (i == extrinsics.length - 1) {
        const unsub = await extrinsics[i].send((result) => {
          if (result.isError) {
            reject(result.toHuman());
          }
          if (result.isInBlock) {
            console.log(`Last extrinsic submitted`);
            unsub();
            resolve(null);
          }
        });
      } else {
        await extrinsics[i].send();
      }
      if (i % 100 == 0) {
        console.log(`Sending extrinsic: ${i}...`);
      }
    }
    console.log(`Waiting for last extrinsic...`);
  });
};

let globalId = 10000;
export async function customWeb3Request(web3: Web3, method: string, params: any[]) {
  return new Promise<JsonRpcResponse>((resolve, reject) => {
    const id = globalId++;
    console.log(`Sending ${id}`);
    (web3.currentProvider as any).send(
      {
        jsonrpc: "2.0",
        id,
        method,
        params,
      },
      (error: Error | null, result?: JsonRpcResponse) => {
        console.log(`Receiving ${id}`, error);
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
