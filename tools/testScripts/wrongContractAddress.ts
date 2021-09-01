// Scrtipt meant to be used against a running ganache-cli node

import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { ALITH, ALITH_PRIVKEY } from "../test-constants";

const ganacheUrl = "http://localhost:8545";
const wrongPrecompileContractAddress = "0x0000666000000000000000000000000000000800";
const GAS_PRICE = "0x" + (1_000_000_000).toString(16);
// We are trying to call candidate_count() on the staking precompile
const dataCall = `0x4b1c4c29`;
// We are trying to call join_candidate(min,1) on the staking precompile
const dataSend = `0x0a1bff6000000000000000000000000000000000000000000000003635c9adc5dea000000000000000000000000000000000000000000000000000000000000000000001`;

export async function customWeb3Request(web3: Web3, method: string, params: any[]) {
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

async function test() {
  const web3 = new Web3(new Web3.providers.HttpProvider(ganacheUrl));
  const resp = await customWeb3Request(web3, "eth_call", [
    {
      from: ALITH,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: wrongPrecompileContractAddress,
      dataCall,
    },
  ]);
  console.log("RESPONSE CALL");
  console.log(resp);

  const respSend = await customWeb3Request(web3, "eth_call", [
    {
      from: ALITH,
      privateKey: ALITH_PRIVKEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: wrongPrecompileContractAddress,
      data: dataSend,
    },
  ]);
  console.log("RESPONSE SEND");
  console.log(respSend);
  console.log(await web3.eth.getBlock("latest"));
}
test();
