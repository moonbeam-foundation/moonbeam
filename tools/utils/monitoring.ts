import { ApiPromise } from "@polkadot/api";
import { DispatchInfo } from "@polkadot/types/interfaces";
import { Extrinsic } from "./types";

export const monitorBlocks = async (api: ApiPromise) => {
  const maxBlockWeight = api.consts.system.blockWeights.maxBlock.toBigInt();
  const unsubHeads = await api.rpc.chain.subscribeNewHeads(async (lastHeader) => {
    const pendingTx = (await api.rpc.author.pendingExtrinsics()).length;
    const block = (await api.rpc.chain.getBlock(lastHeader.hash)).block;
    const records = await api.query.system.events.at(lastHeader.hash);

    const blockWeight = block.extrinsics.reduce(
      (totalWeight, { method: { method, section } }, index) => {
        // filter the specific events based on the phase and then the
        // index of our extrinsic in the block
        return (
          totalWeight +
          records
            .filter(
              ({ event }) =>
                api.events.system.ExtrinsicSuccess.is(event) ||
                api.events.system.ExtrinsicFailed.is(event)
            )
            .map(({ event: { data, method } }) =>
              method === "ExtrinsicSuccess"
                ? (data[0] as unknown as DispatchInfo)
                : (data[1] as unknown as DispatchInfo)
            )
            .reduce((sum, info) => sum + info.weight.toBigInt(), 0n)
        );
      },
      0n
    );

    console.log(
      `Block ${lastHeader.number.toString().padEnd(4, " ")} [${(
        Number((blockWeight * 100n) / maxBlockWeight) / 100
      )
        .toFixed(2)
        .padStart(5, " ")}%][incl: ${block.extrinsics.length
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
