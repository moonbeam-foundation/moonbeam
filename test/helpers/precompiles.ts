import { DevModeContext, fetchCompiledContract } from "@moonwall/cli";
import { PRECOMPILE_AUTHOR_MAPPING_ADDRESS, createRawTransaction } from "@moonwall/util";
import { encodeFunctionData } from "viem";

export const setKeysThroughPrecompile = async (
  context: DevModeContext,
  account: string,
  privateKey: `0x${string}`,
  keys: string,
  handleFail: boolean = false
) => {
  const { abi: authorMappingAbi } = await fetchCompiledContract("AuthorMapping");

  await context.createBlock(
    createRawTransaction(context, {
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
