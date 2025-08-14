import { type DevModeContext, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { keccak256 } from "viem";
import type { PalletEvmCodeMetadata } from "@polkadot/types/lookup";
import { DUMMY_REVERT_BYTECODE } from "./assets.ts";
import { u8aToHex } from "@polkadot/util";

export interface HeavyContract {
  deployed: boolean;
  account: string;
  key: string;
}

/**
 * @description Deploy multiple contracts to test the EVM storage limit.
 * @param context Context of the test
 * @param count Number of contracts to deploy
 * @returns
 */
export const deployHeavyContracts = async (context: DevModeContext, first = 6000, last = 6999) => {
  // Generate the contract addresses
  const contracts = await Promise.all(
    new Array(last - first + 1).fill(0).map(async (_, i) => {
      const account = `0x${(i + first).toString(16).padStart(40, "0")}`;
      return {
        deployed: false,
        account,
        key: context.polkadotJs().query.evm.accountCodes.key(account),
      };
    })
  );

  // Check which contracts are already deployed
  for (const contract of contracts) {
    contract.deployed =
      (await context.polkadotJs().rpc.state.getStorage(contract.key))!.toString().length > 10;
  }

  // Create the contract code (24kb of zeros)
  const evmCode = `60006000fd${"00".repeat(24_000)}`;
  const codeSize = evmCode.length / 2;
  const storageData = `${context
    .polkadotJs()
    .registry.createType("Compact<u32>", `0x${BigInt(codeSize).toString(16)}`)
    .toHex(true)}${evmCode}`;
  const codeMetadataHash = keccak256(`0x${evmCode}`);
  const mockPalletEvmCodeMetadata: PalletEvmCodeMetadata = context
    .polkadotJs()
    .createType("PalletEvmCodeMetadata", {
      size: codeSize,
      hash: codeMetadataHash,
    });

  // Create the batches of contracts to deploy
  const batches = contracts
    .reduce(
      (acc, value) => {
        if (acc[acc.length - 1].length >= 50) acc.push([]);
        if (!value.deployed) {
          acc[acc.length - 1].push([value.key, storageData]);
          acc[acc.length - 1].push([
            context.polkadotJs().query.evm.accountCodesMetadata.key(value.account),
            u8aToHex(mockPalletEvmCodeMetadata.toU8a()),
          ]);
        }

        return acc;
      },
      [[]] as [string, string][][]
    )
    .filter((batch) => batch.length > 0);

  // Set the storage of the contracts
  let nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });
  for (let i = 0; i < batches.length; i++) {
    const batch = batches[i];
    await context.createBlock([
      context
        .polkadotJs()
        .tx.sudo.sudo(context.polkadotJs().tx.system.setStorage(batch))
        .signAsync(alith, {
          nonce: nonce++,
        }),
    ]);
  }
  return contracts as HeavyContract[];
};

export async function deployedContractsInLatestBlock(context: DevModeContext): Promise<string[]> {
  return (await context.polkadotJs().query.system.events())
    .filter(({ event }) => context.polkadotJs().events.ethereum.Executed.is(event))
    .filter(({ event }) => (event.toHuman() as any)["data"]["exitReason"]["Succeed"] === "Returned")
    .map(({ event }) => (event.toHuman() as any)["data"]["to"]);
}
