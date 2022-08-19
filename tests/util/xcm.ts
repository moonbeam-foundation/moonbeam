import { u8aToHex, BN } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { customWeb3Request } from "./providers";

import { DevTestContext } from "./setup-dev-tests";
import {
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";
import { XcmpMessageFormat } from "@polkadot/types/interfaces";

import { AssetMetadata } from "./assets";

// Creates and returns the tx that overrides the paraHRMP existence
// This needs to be inserted at every block in which you are willing to test
// state changes
// The reason is that set_validation_data inherent overrides it
export function mockHrmpChannelExistanceTx(
  context: DevTestContext,
  para: Number,
  maxCapacity: Number,
  maxTotalSize: Number,
  maxMessageSize: Number
) {
  // This constructs the relevant state to be inserted
  const relevantMessageState = {
    dmqMqcHead: "0x0000000000000000000000000000000000000000000000000000000000000000",
    relayDispatchQueueSize: [0, 0],
    egressChannels: [
      [
        para,
        {
          maxCapacity,
          maxTotalSize,
          maxMessageSize,
          msgCount: 0,
          totalSize: 0,
          mqcHead: null,
        },
      ],
    ],
    ingressChannels: [
      [
        para,
        {
          maxCapacity,
          maxTotalSize,
          maxMessageSize,
          msgCount: 0,
          totalSize: 0,
          mqcHead: null,
        },
      ],
    ],
  };

  const stateToInsert: CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot =
    context.polkadotApi.createType(
      "CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot",
      relevantMessageState
    ) as any;

  // Get keys to modify state
  const module = xxhashAsU8a(new TextEncoder().encode("ParachainSystem"), 128);
  const account_key = xxhashAsU8a(new TextEncoder().encode("RelevantMessagingState"), 128);

  const overallKey = new Uint8Array([...module, ...account_key]);

  return context.polkadotApi.tx.system.setStorage([
    [u8aToHex(overallKey), u8aToHex(stateToInsert.toU8a())],
  ]);
}

export async function registerForeignAsset(
  context: DevTestContext,
  asset: any,
  metadata: AssetMetadata,
  unitsPerSecond?: number,
  numAssetsWeightHint?: number
) {
  unitsPerSecond = unitsPerSecond != null ? unitsPerSecond : 0;
  const {
    result: { events: eventsRegister },
  } = await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.assetManager.registerForeignAsset(asset, metadata, new BN(1), true)
    )
  );
  // Look for assetId in events
  const registeredAssetId = eventsRegister
    .find(({ event: { section } }) => section.toString() === "assetManager")
    .event.data[0].toHex()
    .replace(/,/g, "");

  // setAssetUnitsPerSecond
  const {
    result: { events },
  } = await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(
        asset,
        unitsPerSecond,
        numAssetsWeightHint
      )
    )
  );
  // check asset in storage
  const registeredAsset = (
    (await context.polkadotApi.query.assets.asset(registeredAssetId)) as any
  ).unwrap();
  return {
    registeredAssetId,
    events,
    registeredAsset,
  };
}

export function descendOriginFromAddress(context: DevTestContext, address?: string) {
  const originAddress = address != null ? address : "0x0101010101010101010101010101010101010101";
  const derivedMultiLocation = context.polkadotApi.createType(
    "MultiLocation",
    JSON.parse(
      `{\
              "parents": 1,\
              "interior": {\
                "X2": [\
                  { "Parachain": 1 },\
                  { "AccountKey20": \
                    {\
                      "network": "Any",\
                      "key": "${originAddress}"\
                    } \
                  }\
                ]\
              }\
            }`
    )
  );

  const toHash = new Uint8Array([
    ...new Uint8Array([32]),
    ...new TextEncoder().encode("multiloc"),
    ...derivedMultiLocation.toU8a(),
  ]);

  return {
    originAddress,
    descendOriginAddress: u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20)),
  };
}
export interface RawXcmMessage {
  type: string;
  payload: any;
  format?: string;
}

export function buildXcmpMessage(context: DevTestContext, message: RawXcmMessage): number[] {
  const format = message.format != null ? message.format : "ConcatenatedVersionedXcm";
  const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
    "XcmpMessageFormat",
    format
  ) as any;
  const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
    message.type,
    message.payload
  ) as any;

  return [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
}

export async function injectHrmpMessage(
  context: DevTestContext,
  paraId: number,
  message?: RawXcmMessage
) {
  let totalMessage = message != null ? buildXcmpMessage(context, message) : [];
  // Send RPC call to inject XCM message
  await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [paraId, totalMessage]);
}

export async function injectHrmpMessageAndSeal(
  context: DevTestContext,
  paraId: number,
  message?: RawXcmMessage
) {
  await injectHrmpMessage(context, paraId, message);
  // Create a block in which the XCM will be executed
  await context.createBlock();
}
