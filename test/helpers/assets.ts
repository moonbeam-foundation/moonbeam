import "@moonbeam-network/api-augment/moonbase";
import { u8aToHex } from "@polkadot/util";
import type { DevModeContext } from "@moonwall/cli";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import type { KeyringPair } from "@polkadot/keyring/types";
import { encodeFunctionData, parseAbi } from "viem";
import { alith } from "@moonwall/util";

export const EVM_FOREIGN_ASSETS_PALLET_ACCOUNT = "0x6d6f646c666f7267617373740000000000000000";
export const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

export const DUMMY_REVERT_BYTECODE = "0x60006000fd";
export const RELAY_SOURCE_LOCATION = { Xcm: { parents: 1, interior: "Here" } };
export const RELAY_V3_SOURCE_LOCATION = { V3: { parents: 1, interior: "Here" } } as any;
export const PARA_1000_SOURCE_LOCATION = {
  Xcm: { parents: 1, interior: { X1: { Parachain: 1000 } } },
};
export const PARA_1001_SOURCE_LOCATION = {
  Xcm: { parents: 1, interior: { X1: { Parachain: 1001 } } },
};
export const PARA_2000_SOURCE_LOCATION = {
  Xcm: { parents: 1, interior: { X1: { Parachain: 2000 } } },
};

// XCM V4 Locations
export const ASSET_HUB_PARACHAIN_ID = 1_000;
export const ASSET_HUB_LOCATION = {
  parents: 1,
  interior: { X1: [{ Parachain: ASSET_HUB_PARACHAIN_ID }] },
};
export const RELAY_SOURCE_LOCATION_V4 = { parents: 1, interior: { here: null } };

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

export interface TestAsset {
  // The asset id as required by pallet - moonbeam - foreign - assets
  id: bigint | string;
  // The asset's XCM location
  location: any;
  // The asset's metadata
  metadata: AssetMetadata;
  // The asset's relative price as required by pallet - xcm - weight - trader
  relativePrice?: bigint;
}

export function assetContractAddress(assetId: bigint | string): `0x${string}` {
  // Prefix as defined in pallet - moonbeam - foreign - assets (4 bytes)
  const contractBaseAddress = "0xffffffff";
  // Asset part (padded to 16 bytes)
  const assetAddressBytes = BigInt(assetId).toString(16).padStart(32, "0");
  return `${contractBaseAddress}${assetAddressBytes}`;
}

export const patchLocationV4recursively = (value: any) => {
  let result = value;
  // e.g. Convert this: { X1: { Parachain: 1000 } } to { X1: [ { Parachain: 1000 } ] }
  // Also, will remove the Xcm key if it exists.
  if (result && result.Xcm !== undefined) {
    result = result.Xcm;
  }
  if (result && typeof result === "object") {
    if (Array.isArray(result)) {
      return result.map(patchLocationV4recursively);
    }
    for (const k of Object.keys(result)) {
      if (k === "Concrete" || k === "Abstract") {
        return patchLocationV4recursively(result[k]);
      }
      if (k.match(/^[Xx]\d$/g) && !Array.isArray(result[k])) {
        result[k] = Object.entries(result[k]).map(([k, v]) => ({
          [k]: patchLocationV4recursively(v),
        }));
      } else {
        result[k] = patchLocationV4recursively(result[k]);
      }
    }
  }
  return result;
};

function getSupportedAssetStorageKey(asset: any, context: any) {
  const assetV4 = patchLocationV4recursively(asset);

  const module = xxhashAsU8a(new TextEncoder().encode("XcmWeightTrader"), 128);
  const method = xxhashAsU8a(new TextEncoder().encode("SupportedAssets"), 128);

  const assetLocationU8a = context.polkadotJs().createType("StagingXcmV4Location", assetV4).toU8a();

  const blake2concatStagingXcmV4Location = new Uint8Array([
    ...blake2AsU8a(assetLocationU8a, 128),
    ...assetLocationU8a,
  ]);

  return new Uint8Array([...module, ...method, ...blake2concatStagingXcmV4Location]);
}

/**
 * Adds asset to the pallet - xcm - weight - trader, with the relative provided.
 * If the relative price is 0, the asset will be added with a placeholder price.
 *
 * @param assetLocation XCM v4 location of asset
 * @param relativePrice the pallet requires 18 decimals balance needed to equal 1 unit of native
 *                      asset. For example, if the asset price is twice as low as the GLMR price,
 *                      the relative price should be 500_000_000_000_000_000n.
 *
 * @param context
 */
export async function addAssetToWeightTrader(asset: any, relativePrice: bigint, context: any) {
  const assetV4 = patchLocationV4recursively(asset?.Xcm || asset);

  if (relativePrice === 0n) {
    const addAssetWithPlaceholderPrice = context
      .polkadotJs()
      .tx.sudo.sudo(context.polkadotJs().tx.xcmWeightTrader.addAsset(assetV4, 1n));
    const overallAssetKey = getSupportedAssetStorageKey(assetV4, context);

    const overrideAssetPrice = context.polkadotJs().tx.sudo.sudo(
      context.polkadotJs().tx.system.setStorage([
        [
          u8aToHex(overallAssetKey),
          "0x0100000000000000000000000000000000", // (enabled bool, 0 u128)
        ],
      ])
    );
    const batch = context
      .polkadotJs()
      .tx.utility.batch([addAssetWithPlaceholderPrice, overrideAssetPrice]);

    await context.createBlock(batch, {
      expectEvents: [context.polkadotJs().events.xcmWeightTrader.SupportedAssetAdded],
      allowFailures: false,
    });
  } else {
    await context.createBlock(
      context
        .polkadotJs()
        .tx.sudo.sudo(context.polkadotJs().tx.xcmWeightTrader.addAsset(assetV4, relativePrice)),
      {
        expectEvents: [context.polkadotJs().events.xcmWeightTrader.SupportedAssetAdded],
        allowFailures: false,
      }
    );
  }
}

/**
 *
 * This registers a foreign asset via the moonbeam - foreign - assets pallet.
 * This call will deploy the contract and make the erc20 contract of the asset available
 * in the following address: 0xffffffff + assetId
 *
 * @param context
 * @param assetId a bigint representing the assetId (it can be arbitrary for tests)
 * @param xcmLocation the XCM location of the asset in Polkadot
 * @param metadata
 * @returns
 */
export async function registerForeignAsset(
  context: DevModeContext,
  assetId: bigint,
  xcmLocation: any,
  metadata: AssetMetadata
) {
  const { decimals, name, symbol } = metadata;

  // Sanitize Xcm Location
  const xcmLoc = patchLocationV4recursively(xcmLocation);

  const { result } = await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(
        context
          .polkadotJs()
          .tx.evmForeignAssets.createForeignAsset(assetId, xcmLoc, decimals, symbol, name)
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
  sudoAccount: KeyringPair,
  account: `0x${string}`
) {
  const api = context.polkadotJs();

  const xcmTransaction = {
    V2: {
      gas_limit: 160_000n,
      action: {
        Call: assetContractAddress(assetId),
      },
      value: 0n,
      input: encodeFunctionData({
        abi: parseAbi(["function mintInto(address, uint256)"]),
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

/* Registers foreign asset and calls mock asset balance */
export async function registerAndFundAsset(
  context: any,
  asset: TestAsset,
  amount: bigint,
  address: `0x${string}`,
  addToWeightTrader = true
) {
  const result = await registerForeignAsset(
    context,
    BigInt(asset.id),
    asset.location,
    asset.metadata
  );

  if (addToWeightTrader) {
    await addAssetToWeightTrader(asset.location, asset.relativePrice || 0n, context);
  }

  await mockAssetBalance(context, amount, BigInt(asset.id), alith, address);

  return result;
}
