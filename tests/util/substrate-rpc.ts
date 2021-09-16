import { AddressOrPair, ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { DevTestContext } from "./setup-dev-tests";
import { InternalParaTestContext } from "./setup-para-tests";

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

export const createBlockWithExtrinsicParachain = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: InternalParaTestContext,
  sender: AddressOrPair,
  polkadotCall: Call
) => {
  // This should return a string, but is a bit complex to handle type properly so any will suffice
  const extrinsicHash = (await polkadotCall.signAndSend(sender)) as any;

  // We create the block which is containing the extrinsic
  //const blockResult = await context.createBlock();

  // We retrieve the block (including the extrinsics)
  const blockData = await context.polkadotApiParaone.rpc.chain.getBlock();
  console.log("blockData.hash", blockData.hash);
  console.log("blockData", blockData.toHuman());

  // We retrieve the events for that block
  const allRecords = await context.polkadotApiParaone.query.system.events.at(blockData.hash);
  console.log("ok");

  const extrinsicIndex = blockData.block.extrinsics.findIndex(
    (ext) => ext.hash.toHex() == extrinsicHash
  );
  if (extrinsicIndex < 0) {
    throw new Error(`Extrinsic ${extrinsicHash} is missing in the block ${blockData.hash}`);
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
