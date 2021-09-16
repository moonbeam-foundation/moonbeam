import { ApiPromise } from "@polkadot/api";
import { AddressOrPair, ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { u8aToHex } from "@polkadot/util";
import { DevTestContext } from "./setup-dev-tests";
import { InternalParaTestContext } from "./setup-para-tests";

const MAX_NUMBER_TRY = 10;

export const createBlockWithExtrinsic = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  sender: AddressOrPair,
  polkadotCall: Call
) => {
  // This should return a string, but is a bit complex to handle type properly so any will suffice
  const extrinsicHash = (await polkadotCall.signAndSend(sender)) as any;

  // We create the block which is containing the extrinsic
  const blockResult = await context.createBlock();

  // We retrieve the events for that block
  const allRecords = await context.polkadotApi.query.system.events.at(blockResult.block.hash);

  // We retrieve the block (including the extrinsics)
  const blockData = await context.polkadotApi.rpc.chain.getBlock(blockResult.block.hash);

  const extrinsicIndex = blockData.block.extrinsics.findIndex(
    (ext) => ext.hash.toHex() == extrinsicHash
  );
  if (extrinsicIndex < 0) {
    throw new Error(`Extrinsic ${extrinsicHash} is missing in the block ${blockResult.block.hash}`);
  }
  const extrinsic = blockData.block.extrinsics[extrinsicIndex];

  // We retrieve the events associated with the extrinsic
  const events = allRecords
    .filter(
      ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.toNumber() == extrinsicIndex
    )
    .map(({ event }) => event);

  return { extrinsic, events };
};

//wip
export async function waitOneBlock(api: ApiPromise) {
  return new Promise<void>(async (res) => {
    let count = 0;
    let unsub = await api.derive.chain.subscribeNewHeads(async (header) => {
      console.log(`One block elapsed:#${header.number}: ${header.author}`);
      console.log("header hash", header.hash.toHex());
      // console.log(await api.query.system.events.at(header.hash));
      // (await api.query.system.events.at(header.hash)).forEach((e, i) => {
      //   console.log("event", header.number, header.hash.toHex(), i, e.toHuman());
      // });
      count += 1;
      if (count === 2) {
        unsub();
        res();
      }
    });
  });
}

async function lookForExtrinsicAndEvents(api: ApiPromise, extrinsicHash: Uint8Array) {
  // We retrieve the block (including the extrinsics)
  const signedBlock = await api.rpc.chain.getBlock();
  console.log("signedBlock", signedBlock.toHuman());

  // We retrieve the events for that block
  const allRecords = await api.query.system.events.at(signedBlock.block.header.hash);
  // allRecords.forEach((e) => {
  //   console.log(e.toHuman());
  // });
  // console.log("ok");

  const extrinsicIndex = signedBlock.block.extrinsics.findIndex((ext) => {
    console.log("mmm", ext.hash.toHex(), u8aToHex(extrinsicHash));
    return ext.hash.toHex() == u8aToHex(extrinsicHash);
  });
  if (extrinsicIndex < 0) {
    // throw new Error(
    //   `Extrinsic ${extrinsicHash} is missing in the block ${signedBlock.block.header.hash}`
    // );
    console.log(
      `Extrinsic ${extrinsicHash} is missing in the block ${signedBlock.block.header.hash}`
    );
  }
  const extrinsic = signedBlock.block.extrinsics[extrinsicIndex];

  // We retrieve the events associated with the extrinsic
  const events = allRecords
    .filter(
      ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.toNumber() == extrinsicIndex
    )
    .map(({ event }) => event);
  console.log("FOUND EVENTS", events.length);
  return { events, extrinsic };
}

async function tryLookingForEvents(api: ApiPromise, extrinsicHash: Uint8Array) {
  await waitOneBlock(api);
  let { extrinsic, events } = await lookForExtrinsicAndEvents(api, extrinsicHash);
  if (events.length > 0) {
    return {
      extrinsic,
      events,
    };
  } else {
    return await tryLookingForEvents(api, extrinsicHash);
  }
}

//wip
export const createBlockWithExtrinsicParachain = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: InternalParaTestContext,
  sender: AddressOrPair,
  polkadotCall: Call
) => {
  // This should return a string, but is a bit complex to handle type properly so any will suffice
  const extrinsicHash = (await polkadotCall.signAndSend(sender)) as unknown as Uint8Array;
  console.log("extrinsicHash", extrinsicHash);

  // We create the block which is containing the extrinsic
  //const blockResult = await context.createBlock();
  return await tryLookingForEvents(context.polkadotApiParaone, extrinsicHash);
};
