import "@moonbeam-network/api-augment/moonbase";
import { DevModeContext, fetchCompiledContract } from "@moonwall/cli";
import { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId20 } from "@polkadot/types/interfaces/runtime";
import { StagingXcmV4Location } from "@polkadot/types/lookup";
import { encodeFunctionData, parseAbi } from "viem";

export const EVM_FOREIGN_ASSETS_PALLET_ACCOUNT = "0x6d6f646c666f7267617373740000000000000000";
export const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

export const DUMMY_REVERT_BYTECODE = "0x60006000fd";
export const RELAY_SOURCE_LOCATION = { Xcm: { parents: 1, interior: "Here" } };
export const RELAY_SOURCE_LOCATION2 = { Xcm: { parents: 2, interior: "Here" } };
export const RELAY_V3_SOURCE_LOCATION = { V3: { parents: 1, interior: "Here" } } as any;
export const PARA_1000_SOURCE_LOCATION_V3 = {
  Xcm: { parents: 1, interior: { X1: { Parachain: 1000 } } },
};
export const PARA_2000_SOURCE_LOCATION = {
  Xcm: { parents: 1, interior: { X1: { Parachain: 2000 } } },
};
export const PARA_1001_SOURCE_LOCATION = {
  Xcm: { parents: 1, interior: { X1: { Parachain: 1001 } } },
};

// XCM V4 Locations
export const RELAY_SOURCE_LOCATION_V4 = { Xcm: { parents: 1, interior: { here: null } } };
export const PARA_1000_SOURCE_LOCATION_V4 = {
  Xcm: { parents: 1, interior: { X1: [{ Parachain: 1000 }] } },
};
export interface AssetMetadata {
  name: string;
  symbol: string;
  decimals: bigint;
  isFrozen: boolean;
}

export const relayAssetMetadata: AssetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: 12n,
  isFrozen: false,
};

export function assetContractAddress(assetId: bigint | string): `0x${string}` {
  return `0xffffffff${BigInt(assetId).toString(16)}`;
}

/**
 * This registers a foreign asset via the moonbeam-foreign-assets pallet.
 * This call will deploy the contract and make the erc20 contract of the asset available
 * in the following address: 0xffffffff + metadata.id
 */
export async function registerForeignAsset(
  context: DevModeContext,
  assetId: bigint,
  xcmLocation: any,
  metadata: AssetMetadata
) {
  const { decimals, name, symbol } = metadata;
  const { result } = await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(
        context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, xcmLocation, decimals, symbol, name)
      )
  );

  // Fetch the relevant event
  const event = (result as any).events.find(
    ({ event: { method } }) => method.toString() === "ForeignAssetCreated"
  )!.event;

  // Get relevant info from the asset creation event
  const contractAddress = event.data[0].toHuman().toString();
  const registeredAssetLocation = event.data[2].toString();
  const registeredAssetId = event.data[1].toString();

  return {
    registeredAssetId,
    contractAddress,
    registeredAssetLocation,
    events: (result as any).events || [],
  };
}

export async function foreignAssetBalance(
  context: DevModeContext,
  assetId: bigint,
  account: `0x${string}`
) {
  return (await context.viem().readContract({
    functionName: "balanceOf",
    address: assetContractAddress(assetId),
    args: [account],
    abi: parseAbi(["function balanceOf(address account) view returns (uint256)"]),
  })) as bigint;
}

export async function mockAssetBalance(
  context: DevModeContext,
  assetBalance: bigint,
  assetId: bigint,
  assetLocation: any,
  sudoAccount: KeyringPair,
  account: string | AccountId20
) {
  const api = context.polkadotJs();
  const { abi } = fetchCompiledContract("MyToken");

  // Register the asset first
  await registerForeignAsset(context, assetId, assetLocation, relayAssetMetadata as any);

  const xcmTransaction = {
    V2: {
      gas_limit: 160_000n,
      action: {
        Call: assetContractAddress(assetId),
      },
      value: 0n,
      input: encodeFunctionData({
        abi,
        functionName: "mintInto",
        args: [account, assetBalance],
      }),
      access_list: null,
    },
  };
  await context.createBlock(
    api.tx.sudo
      .sudo(
        api.tx.ethereumXcm.forceTransactAs(EVM_FOREIGN_ASSETS_PALLET_ACCOUNT, xcmTransaction, null)
      )
      .signAsync(sudoAccount)
  );
  return;
}
