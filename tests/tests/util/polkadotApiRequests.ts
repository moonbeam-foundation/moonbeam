import Web3 from "web3";
import { JsonRpcResponse } from "web3-core-helpers";
import { TransactionReceipt } from "web3-core";
import { AbiItem } from "web3-utils";
import { Contract } from "web3-eth-contract";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../constants";
import { ApiPromise } from "@polkadot/api";

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(api: ApiPromise): Promise<number> {
  const startTime: number = Date.now();
  try {
    await api.rpc.engine.createBlock(
      //await customRequest(web3, "engine_createBlock", [
      true,
      true
      //null
    );
    //]);
    // if (response.error) {
    //   console.log("error during block creation");
    //   throw new Error(`Unexpected result: ${JSON.stringify(response)}`);
    // }
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }
  return Date.now() - startTime;
}
