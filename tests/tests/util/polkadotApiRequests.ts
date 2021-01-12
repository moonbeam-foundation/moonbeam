import { ApiPromise } from "@polkadot/api";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../constants";
import { Keyring } from "@polkadot/keyring";

// Create a block and finalize it.
// It will include all previously executed transactions since the last finalized block.
export async function createAndFinalizeBlock(api: ApiPromise): Promise<number> {
  const startTime: number = Date.now();
  const keyring = new Keyring({ type: "ethereum" });
  const testAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
  const set_author = api.tx.authorInherent.setAuthor(GENESIS_ACCOUNT);
  try {
    await set_author.signAndSend(testAccount);
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
