import { DevModeContext, fetchCompiledContract } from "@moonwall/cli";
import {
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  PRECOMPILE_DEMOCRACY_ADDRESS,
  createViemTransaction,
} from "@moonwall/util";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { blake2AsHex } from "@polkadot/util-crypto";
import { encodeFunctionData } from "viem";

export const setKeysThroughPrecompile = async (
  context: DevModeContext,
  account: string,
  privateKey: `0x${string}`,
  keys: string,
  handleFail: boolean = false
) => {
  const { abi: authorMappingAbi } = fetchCompiledContract("AuthorMapping");

  await context.createBlock(
    createViemTransaction(context, {
      from: account,
      privateKey,
      to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
      data: encodeFunctionData({
        abi: authorMappingAbi,
        functionName: "setKeys",
        args: [keys],
      }),
      skipEstimation: handleFail,
    })
  );
};

// Keys used to set author-mapping in the tests
export const originalKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000000000000000000000000000000000000000000000000000002",
];
// Concatenated keys
export const concatOriginalKeys = `0x${originalKeys.map((key) => key.slice(2)).join("")}`;

export const SELECTORS = {
  set_keys: "bcb24ddc",
  remove_keys: "3b6c4284",
};

export const newKeys = [
  "0x0000000000000000000000000000000000000000000000000000000000000003",
  "0x0000000000000000000000000000000000000000000000000000000000000004",
];
export const concatNewKeys = `0x${newKeys.map((key) => key.slice(2)).join("")}`;

export const notePreimagePrecompile = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevModeContext,
  proposal: Call
): Promise<`0x${string}`> => {
  const encodedProposal = proposal.method.toHex();

  const { abi } = fetchCompiledContract("Preimage");
  const data = encodeFunctionData({
    abi,
    functionName: "notePreimage",
    args: [encodedProposal],
  });

  const tx = await createViemTransaction(context, {
    to: PRECOMPILE_DEMOCRACY_ADDRESS,
    gas: 2_000_000n,
    data,
  });

  await context.createBlock(tx);
  return blake2AsHex(encodedProposal);
};
