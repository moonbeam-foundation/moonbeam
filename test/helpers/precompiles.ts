import { DevModeContext, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  PRECOMPILE_PREIMAGE_ADDRESS,
  baltathar,
  charleth,
  createViemTransaction,
} from "@moonwall/util";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { blake2AsHex } from "@polkadot/util-crypto";
import { encodeFunctionData, parseEther } from "viem";
import { expectEVMResult } from "./eth-transactions.js";

export const setAuthorMappingKeysViaPrecompile = async (
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
    to: PRECOMPILE_PREIMAGE_ADDRESS,
    gas: 2_000_000n,
    data,
  });

  await context.createBlock(tx);
  return blake2AsHex(encodedProposal);
};

export async function getAuthorMappingInfo(
  context: DevModeContext,
  authorId: string
): Promise<void | { account: string; deposit: bigint }> {
  const mapping = await context.polkadotJs().query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
}

export const setupPoolWithParticipants = async (context: DevModeContext) => {
  const { contractAddress, abi } = await context.deployContract!("ProxyLeaderDemo");
  expect(contractAddress.length).toBeGreaterThan(3);

  // Adds participants
  for (const [privateKey] of [
    [BALTATHAR_PRIVATE_KEY],
    [CHARLETH_PRIVATE_KEY],
    [DOROTHY_PRIVATE_KEY],
  ]) {
    const rawTxn = createViemTransaction(context, {
      to: contractAddress,
      value: parseEther("1"),
      data: encodeFunctionData({
        abi,
        functionName: "joinPool",
      }),
      privateKey,
    });
    const { result } = await context.createBlock(rawTxn);
    expectEVMResult(result!.events, "Succeed");
  }
  return contractAddress;
};

export async function setupWithParticipants(context: DevModeContext) {
  const { contractAddress } = await context.deployContract!("ProxyCallStakingDemo", {
    gas: 5_000_000n,
    value: parseEther("5"),
  });
  // Add participants
  for (const { account, privateKey } of [
    {
      account: baltathar,
      privateKey: BALTATHAR_PRIVATE_KEY,
    },
    {
      account: charleth,
      privateKey: CHARLETH_PRIVATE_KEY,
    },
  ]) {
    // pre-condition provide staking proxy to contract
    await context.createBlock(
      context.polkadotJs().tx.proxy.addProxy(contractAddress, "Staking", 0).signAsync(account)
    );

    const rawTxn = await context.writeContract!({
      contractAddress,
      contractName: "ProxyCallStakingDemo",
      functionName: "join",
      args: [0],
      privateKey,
    });

    await context.createBlock(rawTxn);
  }
  return contractAddress;
}
