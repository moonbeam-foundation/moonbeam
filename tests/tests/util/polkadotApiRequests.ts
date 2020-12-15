import { ApiPromise } from "@polkadot/api";

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(api: ApiPromise): Promise<number> {
  const startTime: number = Date.now();
  try {
    await api.rpc.engine.createBlock(true, true);
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }
  return Date.now() - startTime;
}
