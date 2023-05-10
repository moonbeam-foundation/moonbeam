import { u8aToHex, BN } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { DevModeContext } from "@moonwall/cli";
import { web3EthCall, customWeb3Request, PRECOMPILE_XCM_UTILS_ADDRESS } from "@moonwall/util";
import {
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  XcmVersionedXcm,
} from "@polkadot/types/lookup";
import { XcmpMessageFormat } from "@polkadot/types/interfaces";
import { getCompiled } from "../helpers/contracts.js";
import { AssetMetadata } from "./assets.js";

const XCM_UTILS_CONTRACT = getCompiled("precompiles/xcm-utils/XcmUtils");
const XCM_UTILSTRANSACTOR_INTERFACE = XCM_UTILS_CONTRACT.contract.abi;

// Creates and returns the tx that overrides the paraHRMP existence
// This needs to be inserted at every block in which you are willing to test
// state changes
// The reason is that set_validation_data inherent overrides it
export function mockHrmpChannelExistanceTx(
  context: DevModeContext,
  para: Number,
  maxCapacity: Number,
  maxTotalSize: Number,
  maxMessageSize: Number
) {
  const api = context.polkadotJs({ type: "moon" });
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
    api.createType(
      "CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot",
      relevantMessageState
    ) as any;

  // Get keys to modify state
  const module = xxhashAsU8a(new TextEncoder().encode("ParachainSystem"), 128);
  const account_key = xxhashAsU8a(new TextEncoder().encode("RelevantMessagingState"), 128);
  // @ts-expect-error
  api.rpc.call(wadawda)
  const overallKey = new Uint8Array([...module, ...account_key]);

  return api.tx.system.setStorage([[u8aToHex(overallKey), u8aToHex(stateToInsert.toU8a())]]);
}

export async function registerForeignAsset(
  context: DevModeContext,
  asset: any,
  metadata: AssetMetadata,
  unitsPerSecond?: number,
  numAssetsWeightHint: number = 0 // TODO: Check this is right
) {
  const api = context.polkadotJs({ type: "moon" });
  unitsPerSecond = unitsPerSecond != null ? unitsPerSecond : 0;
  const {
    result: { events: eventsRegister },
  } = await context.createBlock(
    api.tx.sudo.sudo(api.tx.assetManager.registerForeignAsset(asset, metadata, new BN(1), true))
  );
  // Look for assetId in events
  const registeredAssetId = eventsRegister
    .find(({ event: { section } }) => section.toString() === "assetManager")!
    .event.data[0].toHex()
    .replace(/,/g, "");

  // setAssetUnitsPerSecond
  const {
    result: { events },
  } = await context.createBlock(
    api.tx.sudo.sudo(
      api.tx.assetManager.setAssetUnitsPerSecond(asset, unitsPerSecond, numAssetsWeightHint)
    )
  );
  // check asset in storage
  const registeredAsset = ((await api.query.assets.asset(registeredAssetId)) as any).unwrap();
  return {
    registeredAssetId,
    events,
    registeredAsset,
  };
}

export function descendOriginFromAddress(context: DevModeContext, address?: string) {
  const api = context.polkadotJs({ type: "moon" });
  const originAddress = address != null ? address : "0x0101010101010101010101010101010101010101";
  const derivedMultiLocation = api.createType(
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
    descendOriginAddress: u8aToHex(api.registry.hash(toHash).slice(0, 20)),
  };
}

export function sovereignAccountOfSibling(context: DevModeContext, paraId: number): string {
  return u8aToHex(
    new Uint8Array([
      ...new TextEncoder().encode("sibl"),
      ...context.polkadotJs({ type: "moon" }).createType("u32", paraId).toU8a(),
      ...new Uint8Array(12),
    ])
  );
}

export interface RawXcmMessage {
  type: string;
  payload: any;
  format?: string;
}

export function buildXcmpMessage(context: DevModeContext, message: RawXcmMessage): number[] {
  const format = message.format != null ? message.format : "ConcatenatedVersionedXcm";
  const xcmpFormat: XcmpMessageFormat = context
    .polkadotJs({ type: "moon" })
    .createType("XcmpMessageFormat", format) as any;
  const receivedMessage: XcmVersionedXcm = context
    .polkadotJs({ type: "moon" })
    .createType(message.type, message.payload) as any;

  return [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
}

export async function injectHrmpMessage(
  context: DevModeContext,
  paraId: number,
  message?: RawXcmMessage
) {
  let totalMessage = message != null ? buildXcmpMessage(context, message) : [];
  // Send RPC call to inject XCM message
  await customWeb3Request(context.web3(), "xcm_injectHrmpMessage", [paraId, totalMessage]);
}

// Weight a particular message using the xcm utils precompile
export async function weightMessage(context: DevModeContext, message: XcmVersionedXcm) {
  const result = await web3EthCall(context.web3(), {
    to: PRECOMPILE_XCM_UTILS_ADDRESS,
    data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("weightMessage", [message.toU8a()]),
  });
  console.log("remove");
  console.log(JSON.stringify(result)); // TODO: Remove me
  return result;
}

export async function injectHrmpMessageAndSeal(
  context: DevModeContext,
  paraId: number,
  message?: RawXcmMessage
) {
  await injectHrmpMessage(context, paraId, message);
  // Create a block in which the XCM will be executed
  await context.createBlock();
}

interface XcmFragmentConfig {
  assets: {
    multilocation: {
      parents: number;
      interior: any;
    };
    fungible: bigint;
  }[];
  weight_limit?: BN;
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
      ReserveAssetDeposited: this.config.assets.map(({ multilocation, fungible }) => {
        return {
          id: {
            Concrete: multilocation,
          },
          fun: { Fungible: fungible },
        };
      }, this),
    });
    return this;
  }

  // Add a `WithdrawAsset` instruction
  withdraw_asset(): this {
    this.instructions.push({
      WithdrawAsset: this.config.assets.map(({ multilocation, fungible }) => {
        return {
          id: {
            Concrete: multilocation,
          },
          fun: { Fungible: fungible },
        };
      }, this),
    });
    return this;
  }

  // Add one or more `BuyExecution` instruction
  // if weight_limit is not set in config, then we put unlimited
  buy_execution(fee_index: number = 0, repeat: bigint = 1n): this {
    const weightLimit =
      this.config.weight_limit != null
        ? { Limited: this.config.weight_limit }
        : { Unlimited: null };
    for (var i = 0; i < repeat; i++) {
      this.instructions.push({
        BuyExecution: {
          fees: {
            id: {
              Concrete: this.config.assets[fee_index].multilocation,
            },
            fun: { Fungible: this.config.assets[fee_index].fungible },
          },
          weightLimit: weightLimit,
        },
      });
    }
    return this;
  }

  // Add a `ClaimAsset` instruction
  claim_asset(index: number = 0): this {
    this.instructions.push({
      ClaimAsset: {
        assets: [
          {
            id: {
              Concrete: this.config.assets[index].multilocation,
            },
            fun: { Fungible: this.config.assets[index].fungible },
          },
        ],
        // Ticket seems to indicate the version of the assets
        ticket: {
          parents: 0,
          interior: { X1: { GeneralIndex: 3 } },
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
    let error_instructions: any[] = [];
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
    let appendix_instructions: any[] = [];
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

  // Overrides the weight limit of the first buyExeuction encountered
  // with the measured weight
  async override_weight(context: DevModeContext): Promise<this> {
    const message: XcmVersionedXcm = context
      .polkadotJs({ type: "moon" })
      .createType("XcmVersionedXcm", this.as_v2()) as any;

    const instructions = message.asV2;
    for (var i = 0; i < instructions.length; i++) {
      if (instructions[i].isBuyExecution == true) {
        let newWeight = await weightMessage(context, message);
        this.instructions[i] = {
          BuyExecution: {
            fees: instructions[i].asBuyExecution.fees,
            weightLimit: { Limited: newWeight },
          },
        };
        break;
      }
    }
    return this;
  }
}
