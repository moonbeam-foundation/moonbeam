import { alice, web3, ERC20_BYTECODE, init } from "./init-web3";
import { JsonRpcResponse } from "web3-core-helpers";
import Web3 from "web3";
init("ws://localhost:19933");

export async function customWeb3Request(web3: Web3, method: string, params: any[]) {
  return new Promise<JsonRpcResponse>((resolve, reject) => {
    console.log(
      `sending ${JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
      })}`
    );
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

const main = async () => {
  const txHash = "0xa727bef3dd695943c9232149ee84829b255b187968748183114e60a6fabdea72";
  if (!(await web3.eth.getTransaction(txHash))) {
    console.log(`\nCreating contract using Eth RPC "sendTransaction" from alice`);
    const createTransaction = await alice.signTransaction({
      data: ERC20_BYTECODE,
      value: "0x00",
      gasPrice: web3.utils.toWei("1", "Gwei"),
      gas: "0x100000",
    });
    console.log("Transaction", {
      ...createTransaction,
      rawTransaction: `${createTransaction.rawTransaction.substring(0, 32)}... (${
        createTransaction.rawTransaction.length
      } length)`,
    });

    const createReceipt = await web3.eth.sendSignedTransaction(createTransaction.rawTransaction);
    console.log(
      `Contract deployed at address ${createReceipt.contractAddress} (Tx: ${createReceipt.transactionHash})`
    );
    if (createReceipt.transactionHash != txHash) {
      console.log(`Unexpected tx ${createReceipt.transactionHash} vs ${txHash}`);
      process.exit(1);
    }
  }

  const trace = await customWeb3Request(web3, "debug_traceTransaction", [txHash]);
  console.log(trace);
  process.exit(0);
};

main().catch((err) => {
  console.log("Error", err);
});
