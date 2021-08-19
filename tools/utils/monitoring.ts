import { ApiPromise } from "@polkadot/api";
import { Extrinsic } from "./types";

export const monitorBlocks = async (api: ApiPromise) => {
  const unsubHeads = await api.rpc.chain.subscribeNewHeads(async (lastHeader) => {
    const pendingTx = (await api.rpc.author.pendingExtrinsics()).length;
    const block = (await api.rpc.chain.getBlock(lastHeader.hash)).block;
    console.log(
      `Block ${lastHeader.number.toString().padEnd(4, " ")} [incl: ${block.extrinsics.length
        .toString()
        .padStart(3, " ")} txs - pending ${pendingTx.toString().padStart(5, " ")} tx]`
    );
  });
  return unsubHeads;
};

export const sendAllAndWaitLast = async (extrinsics: Extrinsic[]) => {
  return new Promise(async (resolve, reject) => {
    console.log(`Preparing to send ${extrinsics.length} extrinsics`);
    for (let i = 0; i < extrinsics.length; i++) {
      if (i == extrinsics.length - 1) {
        const unsub = await extrinsics[i].send((result) => {
          if (result.isError) {
            reject(result.toHuman());
          }
          if (result.isInBlock) {
            console.log(`Last extrinsic submitted`);
            unsub();
            resolve(null);
          }
        });
      } else {
        await extrinsics[i].send();
      }
      if (i % 100 == 0) {
        console.log(`Sending extrinsic: ${i}...`);
      }
    }
    console.log(`Waiting for last extrinsic...`);
  });
};
