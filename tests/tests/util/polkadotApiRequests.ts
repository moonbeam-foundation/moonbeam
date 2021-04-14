import { ApiPromise } from "@polkadot/api";
import { BlockHash } from "@polkadot/types/interfaces/chain";

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(
  api: ApiPromise,
  parentHash?: BlockHash,
  finalize: boolean = true
): Promise<[number, BlockHash]> {
  const startTime: number = Date.now();
  let block_hash = undefined;
  try {
    if (parentHash == undefined) {
      block_hash = (await api.rpc.engine.createBlock(true, finalize)).toJSON()["hash"];
    } else {
      block_hash = (await api.rpc.engine.createBlock(true, finalize, parentHash)).toJSON()["hash"];
    }
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }

  return [Date.now() - startTime, block_hash];
}
