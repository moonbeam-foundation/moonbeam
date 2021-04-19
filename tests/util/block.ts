import { ApiPromise } from "@polkadot/api";
import { BlockHash } from "@polkadot/types/interfaces/chain";

export async function createAndFinalizeBlock(
  api: ApiPromise,
  parentHash?: BlockHash,
  finalize: boolean = true
): Promise<{
  duration: number;
  hash: BlockHash;
}> {
  const startTime: number = Date.now();
  let hash = undefined;
  try {
    if (parentHash == undefined) {
      hash = (await api.rpc.engine.createBlock(true, finalize)).toJSON()["hash"];
    } else {
      hash = (await api.rpc.engine.createBlock(true, finalize, parentHash)).toJSON()["hash"];
    }
  } catch (e) {
    console.log("ERROR DURING BLOCK FINALIZATION", e);
  }

  return {
    duration: Date.now() - startTime,
    hash,
  };
}
