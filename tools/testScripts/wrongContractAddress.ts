import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { ALITH, ALITH_PRIVKEY } from "../test-constants";

const ganacheUrl = "http://localhost:8545";
const GANACHE_ADDRESS = "0x483Cd27bc3C0FadC3133ec8aC8eEB4C247794339";
const GANACHE_PRIV_KEY = "0x8fb940aed59b9bd6ad8d27c443de13195b4b3d2f64e145c66d86256b67a71e7e";
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

// Script meant to be used against a running ganache-cli node with `ganache-cli --seed moonbeam`
async function testGanache() {
  const web3 = new Web3(new Web3.providers.HttpProvider(ganacheUrl)); // with ganache

  //call
  const resp = await customWeb3Request(web3, "eth_call", [
    {
      from: ALITH,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: wrongPrecompileContractAddress,
      data: dataCall,
    },
  ]);
  console.log("RESPONSE CALL");
  console.log(resp);

  //send
  const tx = await web3.eth.accounts.signTransaction(
    {
      from: GANACHE_ADDRESS,
      data: dataSend,
      to: wrongPrecompileContractAddress,
      value: "0x00",
      gasPrice: 1_000_000_000,
      gas: "0x100000",
    },
    GANACHE_PRIV_KEY
  );
  const createReceipt = await web3.eth.sendSignedTransaction(tx.rawTransaction);
  console.log("RESPONSE SEND");
  console.log(createReceipt);
}

// Script meant to be used against a running moonbeam dev instance
async function testMoonbeamDev() {
  const web3 = new Web3(`ws://localhost:9944`); // with moonbeam dev

  //call
  const resp = await customWeb3Request(web3, "eth_call", [
    {
      from: ALITH,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: wrongPrecompileContractAddress,
      data: dataCall,
    },
  ]);
  console.log("RESPONSE CALL");
  console.log(resp);

  //send
  const tx = await web3.eth.accounts.signTransaction(
    {
      from: ALITH,
      data: dataSend,
      to: wrongPrecompileContractAddress,
      value: "0x00",
      gasPrice: 1_000_000_000,
      gas: "0x100000",
    },
    ALITH_PRIVKEY
  );
  const createReceipt = await web3.eth.sendSignedTransaction(tx.rawTransaction);
  console.log("RESPONSE SEND");
  console.log(createReceipt);
}
testGanache();
