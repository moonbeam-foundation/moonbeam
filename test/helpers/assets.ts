import "@moonbeam-network/api-augment/moonbase";
import { u128 } from "@polkadot/types";
import { BN, hexToU8a, u8aToHex } from "@polkadot/util";
import { expect, DevModeContext } from "@moonwall/cli";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { KeyringPair } from "@polkadot/keyring/types";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import type { AccountId20 } from "@polkadot/types/interfaces/runtime";
import { encodeFunctionData, parseAbi } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";

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
export const RELAY_SOURCE_LOCATION_V4 = { parents: 1, interior: { here: null } };
export const PARA_1000_SOURCE_LOCATION_V4 = { parents: 1, interior: { X1: [{ Parachain: 1000 }] } };
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

const patchLocationV4recursively = (value: any) => {
  // e.g. Convert this: { X1: { Parachain: 1000 } } to { X1: [ { Parachain: 1000 } ] }
  if (value && typeof value == "object") {
    if (Array.isArray(value)) {
      return value.map(patchLocationV4recursively);
    }
    for (const k of Object.keys(value)) {
      if (k === "Concrete" || k === "Abstract") {
        return patchLocationV4recursively(value[k]);
      }
      if (k.match(/^X\d$/g) && !Array.isArray(value[k])) {
        value[k] = Object.entries(value[k]).map(([k, v]) => ({
          [k]: patchLocationV4recursively(v),
        }));
      } else {
        value[k] = patchLocationV4recursively(value[k]);
      }
    }
  }
  return value;
};

const runtimeApi = {
  runtime: {
    XcmPaymentApi: [
      {
        methods: {
          query_acceptable_payment_assets: {
            description: "The API to query acceptable payment assets",
            params: [
              {
                name: "version",
                type: "u32",
              },
            ],
            type: "Result<Vec<XcmVersionedAssetId>, XcmPaymentApiError>",
          },
          query_weight_to_asset_fee: {
            description: "",
            params: [
              {
                name: "weight",
                type: "WeightV2",
              },
              {
                name: "asset",
                type: "XcmVersionedAssetId",
              },
            ],
            type: "Result<u128, XcmPaymentApiError>",
          },
          query_xcm_weight: {
            description: "",
            params: [
              {
                name: "message",
                type: "XcmVersionedXcm",
              },
            ],
            type: "Result<WeightV2, XcmPaymentApiError>",
          },
          query_delivery_fees: {
            description: "",
            params: [
              {
                name: "destination",
                type: "XcmVersionedLocation",
              },
              {
                name: "message",
                type: "XcmVersionedXcm",
              },
            ],
            type: "Result<XcmVersionedAssets, XcmPaymentApiError>",
          },
        },
        version: 1,
      },
    ],
    XcmWeightTrader: [
      {
        methods: {
          add_asset: {
            description: "Add an asset to the supported assets",
            params: [
              {
                name: "asset",
                type: "XcmVersionedAssetId",
              },
              {
                name: "relative_price",
                type: "u128",
              },
            ],
            type: "Result<(), XcmPaymentApiError>",
          },
        },
        version: 1,
      },
    ],
  },
  types: {
    XcmPaymentApiError: {
      _enum: {
        Unimplemented: "Null",
        VersionedConversionFailed: "Null",
        WeightNotComputable: "Null",
        UnhandledXcmVersion: "Null",
        AssetNotFound: "Null",
      },
    },
  },
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

function getSupportedAssedStorageKey(asset: any, context: any) {
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

export async function addAssetToWeightTrader(asset: any, relativePrice: number, context: any) {
  const assetV4 = patchLocationV4recursively(asset.Xcm);

  if (relativePrice == 0) {
    const addAssetWithPlaceholderPrice = context
      .polkadotJs()
      .tx.sudo.sudo(context.polkadotJs().tx.xcmWeightTrader.addAsset(assetV4, 1n));
    const overallAssetKey = getSupportedAssedStorageKey(assetV4, context);

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

  const polkadotJs = await ApiPromise.create({
    provider: new WsProvider(`ws://localhost:${process.env.MOONWALL_RPC_PORT}/`),
    ...runtimeApi,
  });

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
      expectEvents: [context.polkadotJs().events.xcmWeightTrader.SupportedAssetAdded],
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
  // Register the asset
  await registerForeignAsset(context, assetId, RELAY_SOURCE_LOCATION, relayAssetMetadata);

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
  const evmCodeAssetKey = api.query.evm.accountCodes.key("0xFfFFfFff" + assetId.toHex().slice(2));

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
        ])
      )
      .signAsync(sudoAccount)
  );
  return;
}
