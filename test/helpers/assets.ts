import "@moonbeam-network/api-augment/moonbase";
import type { u128 } from "@polkadot/types";
import { BN, hexToU8a, u8aToHex } from "@polkadot/util";
import { expect, type DevModeContext } from "@moonwall/cli";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import type { KeyringPair } from "@polkadot/keyring/types";
import type {
  PalletAssetsAssetAccount,
  PalletAssetsAssetDetails,
  PalletEvmCodeMetadata,
} from "@polkadot/types/lookup";
import type { AccountId20 } from "@polkadot/types/interfaces/runtime";
import { encodeFunctionData, parseAbi, keccak256 } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { alith } from "@moonwall/util";

export const EVM_FOREIGN_ASSETS_PALLET_ACCOUNT = "0x6d6f646c666f7267617373740000000000000000";
export const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

export const DUMMY_REVERT_BYTECODE = "0x60006000fd";
export const RELAY_SOURCE_LOCATION = { Xcm: { parents: 1, interior: "Here" } };
export const RELAY_SOURCE_LOCATION2 = { Xcm: { parents: 2, interior: "Here" } };
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
export const RELAY_SOURCE_LOCATION_V4 = { parents: 1, interior: { here: null } };
export const PARA_1000_SOURCE_LOCATION_V4 = { parents: 1, interior: { X1: [{ Parachain: 1000 }] } };
export const PARA_1001_SOURCE_LOCATION_V4 = { parents: 1, interior: { X1: [{ Parachain: 1001 }] } };

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

export async function calculateRelativePrice(
  context: any,
  unitsPerSecond: number
): Promise<bigint> {
  if (unitsPerSecond === 0) {
    return 0n;
  }

  const WEIGHT_REF_TIME_PER_SECOND = 1_000_000_000_000;
  const weight = {
    refTime: WEIGHT_REF_TIME_PER_SECOND,
    proofSize: 0,
  };

  const nativeAmountPerSecond = await context
    .polkadotJs()
    .tx.transactionPaymentApi.queryWeightToFee(weight);

  const relativePriceDecimals = new BN(18);
  const relativePrice = nativeAmountPerSecond
    .mul(new BN(10).pow(relativePriceDecimals))
    .div(new BN(unitsPerSecond));

  return relativePrice;
}

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
  const assetV4 = patchLocationV4recursively(asset.Xcm);

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

// This registers an old foreign asset via the asset-manager pallet.
// DEPRECATED: Please don't use for new tests
export async function registerOldForeignAsset(
  context: DevModeContext,
  asset: any,
  metadata: AssetMetadata,
  unitsPerSecond?: number,
  numAssetsWeightHint?: number
) {
  const { result } = await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(
        context.polkadotJs().tx.assetManager.registerForeignAsset(asset, metadata, new BN(1), true)
      )
  );

  const WEIGHT_REF_TIME_PER_SECOND = 1_000_000_000_000;
  const weight = {
    refTime: WEIGHT_REF_TIME_PER_SECOND,
    proofSize: 0,
  };

  const nativeAmountPerSecond = await context
    .polkadotJs()
    .call.transactionPaymentApi.queryWeightToFee(weight);

  const relativePriceDecimals = new BN(18);
  const relativePrice = nativeAmountPerSecond
    .mul(new BN(10).pow(relativePriceDecimals))
    .div(unitsPerSecond ? new BN(unitsPerSecond) : new BN(1));

  const assetV4 = patchLocationV4recursively(asset.Xcm);
  const { result: result2 } = await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(context.polkadotJs().tx.xcmWeightTrader.addAsset(assetV4, relativePrice)),
    {
      expectEvents: [context.polkadotJs().events.xcmWeightTrader.SupportedAssetAdded as any],
      allowFailures: false,
    }
  );

  // If no unitspersecond is provided, we add the asset to the supported assets
  // and force-set the relative price to 0
  if (unitsPerSecond == null) {
    const module = xxhashAsU8a(new TextEncoder().encode("XcmWeightTrader"), 128);
    const method = xxhashAsU8a(new TextEncoder().encode("SupportedAssets"), 128);

    const assetLocationU8a = context
      .polkadotJs()
      .createType("StagingXcmV4Location", assetV4)
      .toU8a();

    const blake2concatStagingXcmV4Location = new Uint8Array([
      ...blake2AsU8a(assetLocationU8a, 128),
      ...assetLocationU8a,
    ]);

    const overallAssetKey = new Uint8Array([
      ...module,
      ...method,
      ...blake2concatStagingXcmV4Location,
    ]);

    await context.createBlock(
      context.polkadotJs().tx.sudo.sudo(
        context.polkadotJs().tx.system.setStorage([
          [
            u8aToHex(overallAssetKey),
            "0x0100000000000000000000000000000000", // (enabled bool, 0 u128)
          ],
        ])
      )
    );
  }

  const registeredAssetId = result!.events
    .find(({ event: { section } }) => section.toString() === "assetManager")!
    .event.data[0].toHex()
    .replace(/,/g, "");

  // check asset in storage
  const registeredAsset = (
    (await context.polkadotJs().query.assets.asset(registeredAssetId)) as any
  ).unwrap();
  return {
    registeredAssetId,
    events: result2!.events,
    registeredAsset,
  };
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

// Mock balance for old foreign assets
// DEPRECATED: Please don't use for new tests
export async function mockOldAssetBalance(
  context: DevModeContext,
  assetBalance: PalletAssetsAssetAccount,
  assetDetails: PalletAssetsAssetDetails,
  sudoAccount: KeyringPair,
  assetId: u128,
  account: string | AccountId20,
  is_sufficient = false
) {
  const api = context.polkadotJs();
  // Register the asset
  await context.createBlock(
    api.tx.sudo
      .sudo(
        api.tx.assetManager.registerForeignAsset(
          RELAY_SOURCE_LOCATION,
          relayAssetMetadata,
          new BN(1),
          is_sufficient
        )
      )
      .signAsync(sudoAccount)
  );

  const assets = await api.query.assetManager.assetIdType(assetId);
  // make sure we created it
  expect(assets.unwrap().asXcm.parents.toNumber()).to.equal(1);

  // Get keys to modify balance
  const module = xxhashAsU8a(new TextEncoder().encode("Assets"), 128);
  const account_key = xxhashAsU8a(new TextEncoder().encode("Account"), 128);
  const blake2concatAssetId = new Uint8Array([
    ...blake2AsU8a(assetId.toU8a(), 128),
    ...assetId.toU8a(),
  ]);

  const blake2concatAccount = new Uint8Array([
    ...blake2AsU8a(hexToU8a(account.toString()), 128),
    ...hexToU8a(account.toString()),
  ]);
  const overallAccountKey = new Uint8Array([
    ...module,
    ...account_key,
    ...blake2concatAssetId,
    ...blake2concatAccount,
  ]);

  // Get keys to modify total supply & dummyCode (TODO: remove once dummy code inserted by node)
  const assetKey = xxhashAsU8a(new TextEncoder().encode("Asset"), 128);
  const overallAssetKey = new Uint8Array([...module, ...assetKey, ...blake2concatAssetId]);
  const evmCodeAssetKey = api.query.evm.accountCodes.key(`0xFfFFfFff${assetId.toHex().slice(2)}`);
  const evmCodesMetadataAssetKey = api.query.evm.accountCodesMetadata.key(
    `0xFfFFfFff${assetId.toHex().slice(2)}`
  );

  const codeSize = DUMMY_REVERT_BYTECODE.slice(2).length / 2;
  const codeMetadataHash = keccak256(DUMMY_REVERT_BYTECODE);
  const mockPalletEvmCodeMetadata: PalletEvmCodeMetadata = context
    .polkadotJs()
    .createType("PalletEvmCodeMetadata", {
      size: codeSize,
      hash: codeMetadataHash,
    });

  await context.createBlock(
    api.tx.sudo
      .sudo(
        api.tx.system.setStorage([
          [u8aToHex(overallAccountKey), u8aToHex(assetBalance.toU8a())],
          [u8aToHex(overallAssetKey), u8aToHex(assetDetails.toU8a())],
          [
            evmCodeAssetKey,
            `0x${((DUMMY_REVERT_BYTECODE.length - 2) * 2)
              .toString(16)
              .padStart(2)}${DUMMY_REVERT_BYTECODE.slice(2)}`,
          ],
          [evmCodesMetadataAssetKey, u8aToHex(mockPalletEvmCodeMetadata.toU8a())],
        ])
      )
      .signAsync(sudoAccount)
  );
  return;
}
