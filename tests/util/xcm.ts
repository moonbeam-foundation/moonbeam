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

interface XcmFragmentConfig {
  fees: {
    multilocation: any[];
    fungible: bigint;
  };
  weight_limit: BN;
  descend_origin?: string;
  beneficiary?: string;
}

export class XcmFragment {
  config: XcmFragmentConfig;
  instructions: any[];

  constructor(config: XcmFragmentConfig) {
    this.config = config;
    this.instructions = [];
  }

  // Add a `ReserveAssetDeposited` instruction
  reserve_asset_deposited(): this {
    this.instructions.push({
      ReserveAssetDeposited: this.config.fees.multilocation.map((multilocation) => {
        return {
          id: {
            Concrete: multilocation,
          },
          fun: { Fungible: this.config.fees.fungible },
        };
      }, this),
    });
    return this;
  }

  // Add a `WithdrawAsset` instruction
  withdraw_asset(): this {
    this.instructions.push({
      WithdrawAsset: this.config.fees.multilocation.map((multilocation) => {
        return {
          id: {
            Concrete: multilocation,
          },
          fun: { Fungible: this.config.fees.fungible },
        };
      }, this),
    });
    return this;
  }

  // Add a `BuyExecution` instruction
  buy_execution(multilocation_index: number = 0): this {
    this.instructions.push({
      BuyExecution: {
        fees: {
          id: {
            Concrete: this.config.fees.multilocation[multilocation_index],
          },
          fun: { Fungible: this.config.fees.fungible },
        },
        weightLimit: { Limited: this.config.weight_limit },
      },
    });
    return this;
  }

  // Add a `ClaimAsset` instruction
  claim_asset(): this {
    this.instructions.push({
      ClaimAsset: {
        assets: [
          {
            id: {
              Concrete: this.config.fees.multilocation[0],
            },
            fun: { Fungible: this.config.fees.fungible },
          },
        ],
        // Ticket seems to indicate the version of the assets
        ticket: {
          parents: 0,
          interior: { X1: { GeneralIndex: 2 } },
        },
      },
    });
    return this;
  }

  // Add a `ClearOrigin` instruction
  clear_origin(repeat: bigint = 1n): this {
    for (var i = 0; i < repeat; i++) {
      this.instructions.push({ ClearOrigin: null as any });
    }
    return this;
  }

  // Add a `DescendOrigin` instruction
  descend_origin(): this {
    if (this.config.descend_origin != null) {
      this.instructions.push({
        DescendOrigin: {
          X1: {
            AccountKey20: {
              network: "Any",
              key: this.config.descend_origin,
            },
          },
        },
      });
    } else {
      console.warn("!Building a DescendOrigin instruction without a configured descend_origin");
    }
    return this;
  }

  // Add a `DepositAsset` instruction
  deposit_asset(max_assets: bigint = 1n): this {
    if (this.config.beneficiary == null) {
      console.warn("!Building a DepositAsset instruction without a configured beneficiary");
    }
    this.instructions.push({
      DepositAsset: {
        assets: { Wild: "All" },
        maxAssets: max_assets,
        beneficiary: {
          parents: 0,
          interior: { X1: { AccountKey20: { network: "Any", key: this.config.beneficiary } } },
        },
      },
    });
    return this;
  }

  // Add a `SetErrorHandler` instruction, appending all the nested instructions
  set_error_handler_with(callbacks: Function[]): this {
    let error_instructions = [];
    callbacks.forEach((cb) => {
      cb.call(this);
      // As each method in the class pushes to the instruction stack, we pop
      error_instructions.push(this.instructions.pop());
    });
    this.instructions.push({
      SetErrorHandler: error_instructions,
    });
    return this;
  }

  // Add a `SetAppendix` instruction, appending all the nested instructions
  set_appendix_with(callbacks: Function[]): this {
    let appendix_instructions = [];
    callbacks.forEach((cb) => {
      cb.call(this);
      // As each method in the class pushes to the instruction stack, we pop
      appendix_instructions.push(this.instructions.pop());
    });
    this.instructions.push({
      SetAppendix: appendix_instructions,
    });
    return this;
  }

  // Add a `Trap` instruction
  trap(): this {
    this.instructions.push({
      Trap: 0,
    });
    return this;
  }

  // Utility function to support functional style method call chaining bound to `this` context
  with(callback: Function): this {
    return callback.call(this);
  }

  // Pushes the given instruction
  push_any(instruction: any): this {
    this.instructions.push(instruction);
    return this;
  }

  // Returns a V2 fragment payload
  as_v2(): any {
    return {
      V2: this.instructions,
    };
  }
}
