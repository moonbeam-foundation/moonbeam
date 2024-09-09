import { DevModeContext } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { PalletEvmCodeMetadata } from "@polkadot/types/lookup";
import { u8aToHex } from "@polkadot/util";
import { keccak256 } from "viem";

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
        codesMetadataKey: context.polkadotJs().query.evm.accountCodesMetadata.key(account),
      };
    })
  );

  // Check which contracts are already deployed
  for (const contract of contracts) {
    contract.deployed =
      (await context.polkadotJs().rpc.state.getStorage(contract.key))!.toString().length > 10;
  }

  // Create the contract code (24kb of zeros)
  const evmCode = `60006000fd${"0".repeat(24_000 * 2)}`;
  const storageData = `${context
    .polkadotJs()
    .registry.createType("Compact<u32>", `0x${BigInt((evmCode.length + 1) * 2).toString(16)}`)
    .toHex(true)}${evmCode}`;
  const codeSize = evmCode.length / 2;
  const codeMetadataHash = keccak256(("0x" + evmCode) as `0x${string}`);
  const mockPalletEvmCodeMetadata: PalletEvmCodeMetadata = context
    .polkadotJs()
    .createType("PalletEvmCodeMetadata", {
      size: codeSize,
      hash: codeMetadataHash,
    });

  // Create the batchs of contracts to deploy
  const batchs = contracts
    .reduce(
      (acc, value) => {
        if (acc[acc.length - 1].length >= 30) acc.push([]);
        if (!value.deployed) {
          acc[acc.length - 1].push([value.key, storageData]);
          acc[acc.length - 1].push([
            value.codesMetadataKey,
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
  for (let i = 0; i < batchs.length; i++) {
    const batch = batchs[i];
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
