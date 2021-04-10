import { ApiPromise } from "@polkadot/api";
import { BlockHash } from "@polkadot/types/interfaces/chain";

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(
  api: ApiPromise,
  parentHash?: BlockHash,
  finalize: boolean = true
): Promise<number> {
  const startTime: number = Date.now();
  try {
    if (parentHash == undefined) {
      await api.rpc.engine.createBlock(true, finalize);
    } else {
      await api.rpc.engine.createBlock(true, finalize, parentHash);
    }
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }
  return Date.now() - startTime;
}
