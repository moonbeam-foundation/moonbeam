import { ApiPromise } from "@polkadot/api";
import { GENESIS_ACCOUNT, TEST_ACCOUNT } from "../constants";

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(api: ApiPromise): Promise<number> {
  const startTime: number = Date.now();
  const set_author = api.tx.authorInherent.setAuthor(GENESIS_ACCOUNT.substring(2));
  try {
    await set_author.send();
  } catch (e) {
    console.log("ERROR DURING SETTING AUTHOR", e);
  }
  try {
    await api.rpc.engine.createBlock(true, true);
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }
  return Date.now() - startTime;
}
