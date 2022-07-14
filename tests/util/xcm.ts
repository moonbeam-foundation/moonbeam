import { u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";

import { DevTestContext } from "./setup-dev-tests";

import type { CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot } from "@polkadot/types/lookup";

// Creates and returns the tx that overrides the paraHRMP existance
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
  let module = xxhashAsU8a(new TextEncoder().encode("ParachainSystem"), 128);
  let account_key = xxhashAsU8a(new TextEncoder().encode("RelevantMessagingState"), 128);

  let overallKey = new Uint8Array([...module, ...account_key]);

  return context.polkadotApi.tx.system.setStorage([
    [u8aToHex(overallKey), u8aToHex(stateToInsert.toU8a())],
  ]);
}
