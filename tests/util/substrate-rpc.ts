import { ApiPromise } from "@polkadot/api";
import { AddressOrPair, ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { GenericExtrinsic } from "@polkadot/types/extrinsic";
import { AnyTuple } from "@polkadot/types/types";
import { Event } from "@polkadot/types/interfaces";
import { u8aToHex } from "@polkadot/util";
import { DevTestContext } from "./setup-dev-tests";
const debug = require("debug")("test:substrateEvents");

// DEV LOCAL TESTING

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

// LAUNCH BASED NETWORK TESTING (PARA TESTS)

export async function waitOneBlock(api: ApiPromise, numberOfBlocks: number = 1) {
  return new Promise<void>(async (res) => {
    let count = 0;
    let unsub = await api.derive.chain.subscribeNewHeads(async (header) => {
      console.log(`One block elapsed : #${header.number}: author : ${header.author}`);
      count += 1;
      if (count === 1 + numberOfBlocks) {
        unsub();
        res();
      }
    });
  });
}

// Log relay/parachain new blocks and events
export async function logEvents(api: ApiPromise, name: string) {
  api.derive.chain.subscribeNewHeads(async (header) => {
    debug(
      `------------- ${name} BLOCK#${header.number}: author ${header.author}, hash ${header.hash}`
    );
    (await api.query.system.events.at(header.hash)).forEach((e, i) => {
      debug(
        `${name} Event :`,
        i,
        header.hash.toHex(),
        (e.toHuman() as any).event.section,
        (e.toHuman() as any).event.method
      );
    });
  });
}

async function lookForExtrinsicAndEvents(api: ApiPromise, extrinsicHash: Uint8Array) {
  // We retrieve the block (including the extrinsics)
  const signedBlock = await api.rpc.chain.getBlock();

  // We retrieve the events for that block
  const allRecords = await api.query.system.events.at(signedBlock.block.header.hash);

  const extrinsicIndex = signedBlock.block.extrinsics.findIndex((ext) => {
    return ext.hash.toHex() == u8aToHex(extrinsicHash);
  });
  if (extrinsicIndex < 0) {
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

export const createBlockWithExtrinsicParachain = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  api: ApiPromise,
  sender: AddressOrPair,
  polkadotCall: Call
): Promise<{ extrinsic: GenericExtrinsic<AnyTuple>; events: Event[] }> => {
  console.log("-------------- EXTRINSIC CALL -------------------------------");
  // This should return a Uint8Array
  const extrinsicHash = (await polkadotCall.signAndSend(sender)) as unknown as Uint8Array;

  // We create the block which is containing the extrinsic
  //const blockResult = await context.createBlock();
  return await tryLookingForEvents(api, extrinsicHash);
};
