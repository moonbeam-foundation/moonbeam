import { type DevModeContext, customDevRpcRequest, expect } from "@moonwall/cli";
import { alith, ALITH_ADDRESS, baltathar } from "@moonwall/util";
import type { DispatchError, XcmpMessageFormat } from "@polkadot/types/interfaces";
import type {
  CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot,
  XcmV3JunctionNetworkId,
  XcmVersionedXcm,
  PalletMessageQueueEvent,
} from "@polkadot/types/lookup";
import { type BN, stringToU8a, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { RELAY_V3_SOURCE_LOCATION } from "./assets.js";
import { expectSubstrateEvent } from "./expect.ts";

// Creates and returns the tx that overrides the paraHRMP existence
// This needs to be inserted at every block in which you are willing to test
// state changes
// The reason is that set_validation_data inherent overrides it
export function mockHrmpChannelExistanceTx(
  context: DevModeContext,
  para: number,
  maxCapacity: number,
  maxTotalSize: number,
  maxMessageSize: number
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
    context
      .polkadotJs()
      .createType(
        "CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot",
        relevantMessageState
      ) as any;

  // Get keys to modify state
  const module = xxhashAsU8a(new TextEncoder().encode("ParachainSystem"), 128);
  const account_key = xxhashAsU8a(new TextEncoder().encode("RelevantMessagingState"), 128);

  const overallKey = new Uint8Array([...module, ...account_key]);

  return context
    .polkadotJs()
    .tx.system.setStorage([[u8aToHex(overallKey), u8aToHex(stateToInsert.toU8a())]]);
}

export function descendOriginFromAddress20(
  context: DevModeContext,
  address: `0x${string}` = "0x0101010101010101010101010101010101010101",
  paraId = 1
) {
  const toHash = new Uint8Array([
    ...new TextEncoder().encode("SiblingChain"),
    ...context.polkadotJs().createType("Compact<u32>", paraId).toU8a(),
    ...context
      .polkadotJs()
      .createType("Compact<u32>", "AccountKey20".length + 20)
      .toU8a(),
    ...new TextEncoder().encode("AccountKey20"),
    ...context.polkadotJs().createType("AccountId", address).toU8a(),
  ]);

  return {
    originAddress: address,
    descendOriginAddress: u8aToHex(context.polkadotJs().registry.hash(toHash).slice(0, 20)),
  };
}

export function sovereignAccountOfSibling(context: DevModeContext, paraId: number): string {
  return u8aToHex(
    new Uint8Array([
      ...new TextEncoder().encode("sibl"),
      ...context.polkadotJs().createType("u32", paraId).toU8a(),
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
    .polkadotJs()
    .createType("XcmpMessageFormat", format) as any;
  const receivedMessage: XcmVersionedXcm = context
    .polkadotJs()
    .createType(message.type, message.payload) as any;

  return [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
}

export async function injectHrmpMessage(
  context: DevModeContext,
  paraId: number,
  message?: RawXcmMessage
) {
  const totalMessage = message != null ? buildXcmpMessage(context, message) : [];
  // Send RPC call to inject XCM message
  await customDevRpcRequest("xcm_injectHrmpMessage", [paraId, totalMessage]);
}

export async function injectEncodedHrmpMessageAndSeal(
  context: DevModeContext,
  paraId: number,
  message?: number[]
) {
  // Send RPC call to inject XCM message
  await customDevRpcRequest("xcm_injectHrmpMessage", [paraId, message]);
  // Create a block in which the XCM will be enqueued
  await context.createBlock();
  // The next block will process the hrmp message in the message queue
  return context.createBlock();
}

// Weight a particular message using the xcm utils precompile
export async function weightMessage(context: DevModeContext, message: XcmVersionedXcm) {
  return (await context.readPrecompile!({
    precompileName: "XcmUtils",
    functionName: "weightMessage",
    args: [message.toHex()],
  })) as bigint;
}

// export async function weightMessage(context: DevModeContext, message?: XcmVersionedXcm) {
//   const result = await web3EthCall(context.web3, {
//     to: PRECOMPILE_XCM_UTILS_ADDRESS,
//     data: XCM_UTILSTRANSACTOR_INTERFACE.encodeFunctionData("weightMessage", [message.toU8a()]),
//   });
//   return BigInt(result.result);
// }

export async function injectHrmpMessageAndSeal(
  context: DevModeContext,
  paraId: number,
  message?: RawXcmMessage
) {
  await injectHrmpMessage(context, paraId, message);
  // Create a block in which the XCM will be enqueued.
  //
  // The message will be processed inside on_idle hook of this block
  // using the remaining weight.
  //
  // See https://github.com/paritytech/polkadot-sdk/pull/3844 for more context.
  const { block } = await context.createBlock();
  return block;
}

interface Junction {
  Parachain?: number;
  AccountId32?: { network: "Any" | XcmV3JunctionNetworkId["type"] | null; id: Uint8Array | string };
  AccountIndex64?: { network: "Any" | XcmV3JunctionNetworkId["type"] | null; index: number };
  AccountKey20?: {
    network: "Any" | XcmV3JunctionNetworkId["type"] | null;
    key: Uint8Array | string;
  };
  PalletInstance?: number;
  GeneralIndex?: bigint;
  GeneralKey?: { length: number; data: Uint8Array };
  OnlyChild?: null;
  Plurality?: { id: any; part: any };
  GlobalConsensus?: "Any" | XcmV3JunctionNetworkId["type"];
}

interface Junctions {
  Here?: null;
  X1?: Junction;
  X2?: [Junction, Junction];
  X3?: [Junction, Junction, Junction];
  X4?: [Junction, Junction, Junction, Junction];
  X5?: [Junction, Junction, Junction, Junction, Junction];
  X6?: [Junction, Junction, Junction, Junction, Junction, Junction];
  X7?: [Junction, Junction, Junction, Junction, Junction, Junction, Junction];
  X8?: [Junction, Junction, Junction, Junction, Junction, Junction, Junction, Junction];
}

export interface MultiLocation {
  parents: number;
  interior: Junctions;
}

export interface XcmFragmentConfig {
  assets: {
    multilocation: MultiLocation;
    fungible: bigint;
  }[];
  weight_limit?:
    | BN
    | {
        refTime: BN | number | bigint;
        proofSize: BN | number | bigint;
      };
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
  buy_execution(fee_index = 0, repeat = 1n): this {
    const weightLimit =
      this.config.weight_limit != null
        ? { Limited: this.config.weight_limit }
        : { Unlimited: null };
    for (let i = 0; i < repeat; i++) {
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

  // Add one or more `BuyExecution` instruction
  // if weight_limit is not set in config, then we put unlimited
  refund_surplus(repeat = 1n): this {
    for (let i = 0; i < repeat; i++) {
      this.instructions.push({
        RefundSurplus: null,
      });
    }
    return this;
  }

  // Add a `ClaimAsset` instruction
  claim_asset(index = 0): this {
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
          interior: { X1: { GeneralIndex: 4 } },
        },
      },
    });
    return this;
  }

  // Add a `ClearOrigin` instruction
  clear_origin(repeat = 1n): this {
    for (let i = 0; i < repeat; i++) {
      this.instructions.push({ ClearOrigin: null as any });
    }
    return this;
  }

  // Add a `DescendOrigin` instruction
  descend_origin(network: "Any" | XcmV3JunctionNetworkId["type"] | null = null): this {
    if (this.config.descend_origin != null) {
      this.instructions.push({
        DescendOrigin: {
          X1: {
            AccountKey20: {
              network,
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
  deposit_asset(
    max_assets = 1n,
    network: XcmV3JunctionNetworkId["type"] | null = null,
    beneficiary: MultiLocation | null = null
  ): this {
    if (this.config.beneficiary == null) {
      console.warn("!Building a DepositAsset instruction without a configured beneficiary");
    }
    this.instructions.push({
      DepositAsset: {
        assets: { Wild: { AllCounted: max_assets } },
        beneficiary: beneficiary ?? {
          parents: 0,
          interior: { X1: { AccountKey20: { network, key: this.config.beneficiary } } },
        },
      },
    });
    return this;
  }

  // Add a `DepositAsset` instruction for specific beneficiary and token
  deposit_asset_definite(
    location: any,
    amount: bigint,
    beneficiary: `0x${string}`,
    network: XcmV3JunctionNetworkId["type"] | null = null
  ): this {
    this.instructions.push({
      DepositAsset: {
        assets: {
          Definite: [
            {
              id: {
                Concrete: location,
              },
              fun: {
                Fungible: amount,
              },
            },
          ],
        },
        beneficiary: {
          parents: 0,
          interior: { X1: { AccountKey20: { network, key: beneficiary } } },
        },
      },
    });
    return this;
  }

  // Add a `SetErrorHandler` instruction, appending all the nested instructions
  set_error_handler_with(callbacks: XcmCallback[]): this {
    const error_instructions: any[] = [];
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
  set_appendix_with(callbacks: XcmCallback[]): this {
    const appendix_instructions: any[] = [];
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
  with(callback: (this: this) => void): this {
    callback.call(this);
    return this;
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

  /// XCM V3 calls
  as_v3(): any {
    return {
      V3: this.instructions,
    };
  }

  /// XCM V4 calls
  as_v4(): any {
    const patchLocationV4recursively = (value: any) => {
      // e.g. Convert this: { X1: { Parachain: 1000 } } to { X1: [ { Parachain: 1000 } ] }
      if (value && typeof value === "object") {
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
    return {
      V4: this.instructions.map((inst) => patchLocationV4recursively(inst)),
    };
  }

  // Add a `BurnAsset` instruction
  burn_asset(amount = 0n): this {
    this.instructions.push({
      BurnAsset: this.config.assets.map(({ multilocation, fungible }) => {
        return {
          id: {
            Concrete: multilocation,
          },
          fun: { Fungible: amount === 0n ? fungible : amount },
        };
      }, this),
    });
    return this;
  }

  // Add a `ReportHolding` instruction
  report_holding(
    destination: MultiLocation = { parents: 1, interior: { X1: { Parachain: 1000 } } },
    query_id: number = Math.floor(Math.random() * 1000),
    max_weight: { refTime: bigint; proofSize: bigint } = {
      refTime: 1_000_000_000n,
      proofSize: 1_000_000_000n,
    }
  ): this {
    this.instructions.push({
      ReportHolding: {
        response_info: {
          destination,
          query_id,
          max_weight,
        },
        assets: { Wild: "All" },
      },
    });
    return this;
  }

  // Add a `ExpectAsset` instruction
  expect_asset(): this {
    this.instructions.push({
      ExpectAsset: this.config.assets.map(({ multilocation, fungible }) => {
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

  // Add a `ExpectOrigin` instruction
  expect_origin(
    multilocation: MultiLocation = { parents: 1, interior: { X1: { Parachain: 1000 } } }
  ): this {
    this.instructions.push({
      ExpectOrigin: multilocation,
    });
    return this;
  }

  // Add a `ExpectError` instruction
  expect_error(index = 0, error = "Unimplemented"): this {
    this.instructions.push({
      ExpectError: [index, error],
    });
    return this;
  }

  // Add a `ExpectTransactStatus` instruction
  expect_transact_status(status = "Success"): this {
    this.instructions.push({
      ExpectTransactStatus: status,
    });
    return this;
  }

  // Add a `QueryPallet` instruction
  query_pallet(
    destination: MultiLocation = { parents: 1, interior: { X1: { Parachain: 1000 } } },
    query_id: number = Math.floor(Math.random() * 1000),
    module_name = "pallet_balances",
    max_weight: { refTime: bigint; proofSize: bigint } = {
      refTime: 1_000_000_000n,
      proofSize: 1_000_000_000n,
    }
  ): this {
    this.instructions.push({
      QueryPallet: {
        module_name,
        response_info: {
          destination,
          query_id,
          max_weight,
        },
      },
    });
    return this;
  }

  // Add a `ExpectPallet` instruction
  expect_pallet(
    index = 0,
    name = "Balances",
    module_name = "pallet_balances",
    crate_major = 4,
    min_crate_minor = 0
  ): this {
    this.instructions.push({
      ExpectPallet: {
        index,
        name,
        module_name,
        crate_major,
        min_crate_minor,
      },
    });
    return this;
  }

  // Add a `ReportTransactStatus` instruction
  report_transact_status(
    destination: MultiLocation = { parents: 1, interior: { X1: { Parachain: 1000 } } },
    query_id: number = Math.floor(Math.random() * 1000),
    max_weight: { refTime: bigint; proofSize: bigint } = {
      refTime: 1_000_000_000n,
      proofSize: 1_000_000_000n,
    }
  ): this {
    this.instructions.push({
      ReportTransactStatus: {
        destination,
        query_id,
        max_weight,
      },
    });
    return this;
  }

  // Add a `ClearTransactStatus` instruction
  clear_transact_status(): this {
    this.instructions.push({
      ClearTransactStatus: null as any,
    });
    return this;
  }

  // Add a `UniversalOrigin` instruction
  universal_origin(junction: Junction): this {
    this.instructions.push({
      UniversalOrigin: junction,
    });
    return this;
  }

  // Add a `ExportMessage` instruction
  export_message(
    xcm_hex = "",
    network: "Any" | XcmV3JunctionNetworkId["type"] = "Ethereum",
    destination: Junctions = { X1: { Parachain: 1000 } }
  ): this {
    const callVec = stringToU8a(xcm_hex);
    const xcm = Array.from(callVec);
    this.instructions.push({
      ExportMessage: {
        network,
        destination,
        xcm,
      },
    });
    return this;
  }

  // Add a `LockAsset` instruction
  lock_asset(
    multilocation: MultiLocation = this.config.assets[0].multilocation,
    fungible: bigint = this.config.assets[0].fungible,
    unlocker: MultiLocation = this.config.assets[0].multilocation
  ): this {
    this.instructions.push({
      LockAsset: {
        asset: {
          id: {
            Concrete: multilocation,
          },
          fun: {
            Fungible: fungible,
          },
        },
        unlocker,
      },
    });
    return this;
  }

  // Add a `UnlockAsset` instruction
  unlock_asset(
    multilocation: MultiLocation = this.config.assets[0].multilocation,
    fungible: bigint = this.config.assets[0].fungible,
    target: MultiLocation = this.config.assets[0].multilocation
  ): this {
    this.instructions.push({
      UnlockAsset: {
        asset: {
          id: {
            Concrete: multilocation,
          },
          fun: {
            Fungible: fungible,
          },
        },
        target,
      },
    });
    return this;
  }

  // Add a `NoteUnlockable` instruction
  note_unlockable(
    multilocation: MultiLocation = this.config.assets[0].multilocation,
    fungible: bigint = this.config.assets[0].fungible,
    owner: MultiLocation = this.config.assets[0].multilocation
  ): this {
    this.instructions.push({
      NoteUnlockable: {
        asset: {
          id: {
            Concrete: multilocation,
          },
          fun: {
            Fungible: fungible,
          },
        },
        owner,
      },
    });
    return this;
  }

  // Add a `RequestUnlock` instruction
  request_unlock(
    multilocation: MultiLocation = this.config.assets[0].multilocation,
    fungible: bigint = this.config.assets[0].fungible,
    locker: MultiLocation = this.config.assets[0].multilocation
  ): this {
    this.instructions.push({
      RequestUnlock: {
        asset: {
          id: {
            Concrete: multilocation,
          },
          fun: {
            Fungible: fungible,
          },
        },
        locker,
      },
    });
    return this;
  }

  // Add a `SetFeesMode` instruction
  set_fees_mode(jit_withdraw = true): this {
    this.instructions.push({
      SetFeesMode: { jit_withdraw },
    });
    return this;
  }

  // Add a `SetTopic` instruction
  set_topic(topic = "0xk89103a9CF04c71Dbc94D0b566f7A2"): this {
    this.instructions.push({
      SetTopic: Array.from(stringToU8a(topic)),
    });
    return this;
  }

  // Add a `ClearTopic` instruction
  clear_topic(): this {
    this.instructions.push({
      ClearTopic: null as any,
    });
    return this;
  }

  // Add a `AliasOrigin` instruction
  alias_origin(
    destination: MultiLocation = {
      parents: 1,
      interior: { X1: { Parachain: 1000 } },
    }
  ): this {
    this.instructions.push({
      AliasOrigin: destination,
    });
    return this;
  }

  // Add a `UnpaidExecution` instruction
  unpaid_execution(
    destination: MultiLocation = {
      parents: 1,
      interior: { X1: { Parachain: 1000 } },
    }
  ): this {
    const weight_limit =
      this.config.weight_limit != null
        ? { Limited: this.config.weight_limit }
        : { Unlimited: null };
    this.instructions.push({
      UnpaidExecution: {
        weight_limit,
        check_origin: destination,
      },
    });
    return this;
  }

  // Overrides the weight limit of the first buyExeuction encountered
  // with the measured weight
  async override_weight(context: DevModeContext): Promise<this> {
    const message: XcmVersionedXcm = context
      .polkadotJs()
      .createType("XcmVersionedXcm", this.as_v2()) as any;

    const instructions = message.asV2;
    for (let i = 0; i < instructions.length; i++) {
      if (instructions[i].isBuyExecution === true) {
        const newWeight = await weightMessage(context, message);
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

export const registerXcmTransactorAndContract = async (context: DevModeContext) => {
  await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(context.polkadotJs().tx.xcmTransactor.register(ALITH_ADDRESS, 0))
  );

  await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(
        context
          .polkadotJs()
          .tx.xcmTransactor.setTransactInfo(
            RELAY_V3_SOURCE_LOCATION,
            { refTime: 1, proofSize: 64 * 1024 } as any,
            { refTime: 20_000_000_000, proofSize: 256 * 1024 } as any,
            { refTime: 1, proofSize: 64 * 1024 } as any
          )
      )
  );

  await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(
        context
          .polkadotJs()
          .tx.xcmTransactor.setFeePerSecond(RELAY_V3_SOURCE_LOCATION, 1000000000000n)
      )
  );
};

export const registerXcmTransactorDerivativeIndex = async (context: DevModeContext) => {
  await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(context.polkadotJs().tx.xcmTransactor.register(ALITH_ADDRESS, 0))
  );
};

export const expectXcmEventMessage = async (context: DevModeContext, message: string) => {
  const records = await context.polkadotJs().query.system.events();

  return records
    .filter(({ event }) => context.polkadotJs().events.xcmpQueue.Fail.is(event))
    .some(
      ({ event: { data: eventData } }: { event: { data: any } }) =>
        eventData.error.toString() === message
    );
};

type XcmCallback = (this: XcmFragment) => void;

export const sendCallAsPara = async (
  call: any,
  paraId: number,
  context: DevModeContext,
  fungible = 10_000_000_000_000_000_000n, // Default 10 GLMR
  allowFailure = false,
  opts?: {
    originKind?: string;
  }
) => {
  const getPalletIndex = async (name: string, context: DevModeContext) => {
    const metadata = await context.polkadotJs().rpc.state.getMetadata();
    return metadata.asLatest.pallets
      .find(({ name: palletName }) => palletName.toString() === name)!
      .index.toNumber();
  };

  const encodedCall = call.method.toHex();
  const balancesPalletIndex = await getPalletIndex("Balances", context);

  const QUERY_ID = 43981;

  const xcmMessage = new XcmFragment({
    assets: [
      {
        multilocation: {
          parents: 0,
          interior: {
            X1: { PalletInstance: balancesPalletIndex },
          },
        },
        fungible: fungible,
      },
    ],
    weight_limit: {
      refTime: 40_000_000_000n,
      proofSize: 150_000n,
    },
    beneficiary: sovereignAccountOfSibling(context, paraId),
  })
    .withdraw_asset()
    .buy_execution()
    .push_any({
      Transact: {
        originKind: opts?.originKind ?? "Xcm",
        requireWeightAtMost: {
          refTime: 20_089_165_000n,
          proofSize: 80_000n,
        },
        call: {
          encoded: encodedCall,
        },
      },
    })
    .report_transact_status(
      {
        parents: 1,
        interior: { X1: { Parachain: paraId } },
      },
      QUERY_ID
    )
    .refund_surplus()
    .deposit_asset(1n, null, {
      parents: 1,
      interior: { X1: { Parachain: paraId } },
    })
    .as_v4();

  const mockHrmpExistanceTx = context
    .polkadotJs()
    .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, paraId, 1000, 102400, 102400));
  await mockHrmpExistanceTx.signAndSend(alith);

  // Send an XCM and create block to execute it
  await injectHrmpMessage(context, paraId, {
    type: "XcmVersionedXcm",
    payload: xcmMessage,
  } as RawXcmMessage);

  const blockRes = await context.createBlock([]); // Passing an empty array to get the correct return type

  const event = await expectSubstrateEvent(blockRes, "messageQueue", "Processed");
  // expect(context.polkadotJs().events.messageQueue.Processed.is(event)).to.be.true;
  // Processed.success == true, to check that xcm message was processed successfully
  expect(event.data[3].toJSON()).to.be.true;

  const transactStatusMsg = (
    await context.polkadotJs().query.parachainSystem.hrmpOutboundMessages()
  )[0];

  const transactStatusEncoded = "0x" + transactStatusMsg.data.toHex().slice(4);
  const transactStatusDecoded = context
    .polkadotJs()
    .createType("XcmVersionedXcm", transactStatusEncoded) as XcmVersionedXcm;

  let didSucceed = false;
  let errorName: string | null = null;

  const transactStatusQuery = transactStatusDecoded.asV4[0].asQueryResponse;
  expect(transactStatusQuery.queryId.toNumber()).to.be.eq(QUERY_ID);
  const dispatch = transactStatusQuery.response.asDispatchResult;
  if (dispatch.isSuccess) {
    didSucceed = true;
  } else {
    const error = dispatch.asError;
    const dispatchError = context.polkadotJs().createType("DispatchError", error) as DispatchError;
    if (dispatchError.isModule) {
      const err = context.polkadotJs().registry.findMetaError({
        index: dispatchError.asModule.index,
        error: dispatchError.asModule.error,
      });
      errorName = err.name;
    } else {
      errorName = dispatchError.type;
    }
  }

  if (!allowFailure) {
    expect(errorName).to.be.null;
    expect(didSucceed).to.be.true;
  }

  // this seems to fix an issue where we see SubscribeVersion instead of QueryResponse.
  // a better fix would be set a correct XCM Version in pallet xcm for the parachain,
  // so pallet xcm wouldn't need to send a SubscribeVersion message.
  await context.createBlock();

  return { blockRes, didSucceed, errorName };
};

export const sendCallAsDescendedOrigin = async (
  address: `0x${string}`,
  call: any,
  paraId: number,
  context: DevModeContext,
  fungible = 10_000_000_000_000_000_000n, // Default 10 GLMR
  allowFailure = false
) => {
  const descndedAddress = descendOriginFromAddress20(context, address, paraId);
  const getPalletIndex = async (name: string, context: DevModeContext) => {
    const metadata = await context.polkadotJs().rpc.state.getMetadata();
    return metadata.asLatest.pallets
      .find(({ name: palletName }) => palletName.toString() === name)!
      .index.toNumber();
  };

  const encodedCall = call.method.toHex();
  const balancesPalletIndex = await getPalletIndex("Balances", context);

  const QUERY_ID = 43981;

  const xcmMessage = new XcmFragment({
    assets: [
      {
        multilocation: {
          parents: 0,
          interior: {
            X1: { PalletInstance: balancesPalletIndex },
          },
        },
        fungible: fungible,
      },
    ],
    weight_limit: {
      refTime: 40_000_000_000n,
      proofSize: 150_000n,
    },
    descend_origin: address,
    beneficiary: descndedAddress.descendOriginAddress,
  })
    .withdraw_asset()
    .buy_execution()
    .descend_origin()
    .push_any({
      Transact: {
        originKind: "Xcm",
        requireWeightAtMost: {
          refTime: 20_089_165_000n,
          proofSize: 80_000n,
        },
        call: {
          encoded: encodedCall,
        },
      },
    })
    .report_transact_status(
      {
        parents: 1,
        interior: { X1: { Parachain: paraId } },
      },
      QUERY_ID
    )
    .refund_surplus()
    .deposit_asset(1n, null, {
      parents: 1,
      interior: { X1: { Parachain: paraId } },
    })
    .as_v4();

  const mockHrmpExistanceTx = context
    .polkadotJs()
    .tx.sudo.sudo(mockHrmpChannelExistanceTx(context, paraId, 1000, 102400, 102400));
  await mockHrmpExistanceTx.signAndSend(alith);

  // Send an XCM and create block to execute it
  await injectHrmpMessage(context, paraId, {
    type: "XcmVersionedXcm",
    payload: xcmMessage,
  } as RawXcmMessage);

  const { block } = await context.createBlock();

  const transactStatusMsg = (
    await context.polkadotJs().query.parachainSystem.hrmpOutboundMessages()
  )[0];

  const transactStatusEncoded = "0x" + transactStatusMsg.data.toHex().slice(4);
  const transactStatusDecoded = context
    .polkadotJs()
    .createType("XcmVersionedXcm", transactStatusEncoded) as XcmVersionedXcm;

  let didSucceed = false;
  let errorName: string | null = null;

  if (transactStatusDecoded.asV4[0].isSubscribeVersion) {
    // Successful executions don't generate response messages in the same block
    // instead, they send a subscription message to the destination parachain
    // We assume that the call was successful if we receive a subscription message
    didSucceed = true;
  } else {
    const transactStatusQuery = transactStatusDecoded.asV4[0].asQueryResponse;
    expect(transactStatusQuery.queryId.toNumber()).to.be.eq(QUERY_ID);
    const dispatch = transactStatusQuery.response.asDispatchResult;
    if (dispatch.isSuccess) {
      didSucceed = true;
    } else {
      const error = dispatch.asError;
      const dispatchError = context
        .polkadotJs()
        .createType("DispatchError", error) as DispatchError;
      if (dispatchError.isModule) {
        const err = context.polkadotJs().registry.findMetaError({
          index: dispatchError.asModule.index,
          error: dispatchError.asModule.error,
        });
        errorName = err.name;
      } else {
        errorName = dispatchError.type;
      }
    }
  }

  if (!allowFailure) {
    expect(didSucceed).to.be.true;
  }

  return { block, didSucceed, errorName };
};
