// constants.ts -  Any common values here should be moved to moonwall if suitable

import { DevModeContext } from "@moonwall/cli";
import {
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_GENESIS_LOCK_BALANCE,
  ALITH_GENESIS_RESERVE_BALANCE,
} from "@moonwall/util";

/**
 * Class allowing to store multiple value for a runtime constant based on the runtime version
 */
class RuntimeConstant<T> {
  private values: { [version: number]: T };

  /*
   * Get the expected value for a given runtime version. Lookup for the closest smaller runtime
   */
  get(runtimeVersion: number): T {
    const versions = Object.keys(this.values).map(Number); // slow but easier to maintain
    let value;
    for (let i = 0; i < versions.length; i++) {
      if (versions[i] > runtimeVersion) {
        break;
      }
      value = this.values[versions[i]];
    }
    return value;
  }

  // Builds RuntimeConstant with single or multiple values
  constructor(values: { [version: number]: T } | T) {
    if (values instanceof Object) {
      this.values = values;
    } else {
      this.values = { 0: values };
    }
  }
}

// Crowdloan Constants

export const VESTING_PERIOD = 201600n;
export const RELAYCHAIN_ARBITRARY_ADDRESS_1: string =
  "0x1111111111111111111111111111111111111111111111111111111111111111";
export const RELAYCHAIN_ARBITRARY_ADDRESS_2: string =
  "0x2222222222222222222222222222222222222222222222222222222222222222";

export const ALITH_GENESIS_TRANSFERABLE_COUNT =
  ALITH_GENESIS_FREE_BALANCE + ALITH_GENESIS_RESERVE_BALANCE - ALITH_GENESIS_LOCK_BALANCE;
export const ALITH_GENESIS_TRANSFERABLE_BALANCE =
  ALITH_GENESIS_FREE_BALANCE > ALITH_GENESIS_TRANSFERABLE_COUNT
    ? ALITH_GENESIS_TRANSFERABLE_COUNT
    : ALITH_GENESIS_FREE_BALANCE;

// Precompiles
export const PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS = "0x0000000000000000000000000000000000000817";
export const PRECOMPILE_IDENTITY_ADDRESS = "0x0000000000000000000000000000000000000818";
export const PRECOMPILE_RELAY_DATA_VERIFIER_ADDRESS = "0x0000000000000000000000000000000000000819";

// Fees and gas limits
export const RUNTIME_CONSTANTS = {
  MOONBASE: {
    MIN_FEE_MULTIPLIER: 100_000_000_000_000_000n,
    MAX_FEE_MULTIPLIER: 100_000_000_000_000_000_000_000n,
    MIN_BASE_FEE_IN_WEI: "125000000",
    MAX_BASE_FEE_IN_WEI: "125000000000000",
    TARGET_FILL_PERMILL: new RuntimeConstant({ 3000: 350_000n, 2801: 500_000n, 0: 250_000n }),
    // Deadline for block production in miliseconds
    DEADLINE_MILISECONDS: 2000n,
    // Caclulated as the weight per second by the deadline in seconds
    BLOCK_WEIGHT_LIMIT: 2_000_000_000_000n,
    // Gas limit considering the block utilization threshold (75%)
    GAS_LIMIT: 60_000_000n,
    // Maximum extrinsic weight is taken from the max allowed transaction weight per block (75%),
    // minus the block initialization (10%) and minus the extrinsic base cost.
    EXTRINSIC_GAS_LIMIT: 52_000_000n,
    // Maximum Gas to PoV ratio used in the gasometer
    GAS_PER_POV_BYTES: 16n,
  },
  MOONRIVER: {
    MIN_FEE_MULTIPLIER: 1_000_000_000_000_000_000n,
    MAX_FEE_MULTIPLIER: 100_000_000_000_000_000_000_000n,
    MIN_BASE_FEE_IN_WEI: "1250000000",
    MAX_BASE_FEE_IN_WEI: "125000000000000",
    TARGET_FILL_PERMILL: new RuntimeConstant({ 3000: 350_000n, 2801: 500_000n, 0: 250_000n }),
    // Deadline for block production in miliseconds
    DEADLINE_MILISECONDS: 1000n,
    // Caclulated as the weight per second by the deadline in seconds
    BLOCK_WEIGHT_LIMIT: 1_000_000_000_000n,
    // Gas limit considering the block utilization threshold (75%)
    GAS_LIMIT: 30_000_000n,
    // Maximum extrinsic weight is taken from the max allowed transaction weight per block (75%),
    // minus the block initialization (10%) and minus the extrinsic base cost.
    EXTRINSIC_GAS_LIMIT: 26_000_000n,
    // Maximum Gas to PoV ratio used in the gasometer
    GAS_PER_POV_BYTES: 8n,
  },
  MOONBEAM: {
    MIN_FEE_MULTIPLIER: 1_000_000_000_000_000_000n,
    MAX_FEE_MULTIPLIER: 100_000_000_000_000_000_000_000n,
    MIN_BASE_FEE_IN_WEI: "125000000000",
    MAX_BASE_FEE_IN_WEI: "12500000000000000",
    TARGET_FILL_PERMILL: new RuntimeConstant({ 3000: 350_000n, 2801: 500_000n, 0: 250_000n }),
    // Deadline for block production in miliseconds
    DEADLINE_MILISECONDS: 1000n,
    // Caclulated as the weight per second by the deadline in seconds
    BLOCK_WEIGHT_LIMIT: 1_000_000_000_000n,
    // Gas limit considering the block utilization threshold (75%)
    GAS_LIMIT: 30_000_000n,
    // Maximum extrinsic weight is taken from the max allowed transaction weight per block (75%),
    // minus the block initialization (10%) and minus the extrinsic base cost.
    EXTRINSIC_GAS_LIMIT: 26_000_000n,
    // Maximum Gas to PoV ratio used in the gasometer
    GAS_PER_POV_BYTES: 8n,
  },
} as const;

export const GAS_LIMIT_POV_RATIO = 16;

// Maximum PoV size in bytes allowed by the gasometer for one ethereum transaction
export const MAX_ETH_POV_PER_TX = 3_250_000n;

type ConstantStoreType = (typeof RUNTIME_CONSTANTS)["MOONBASE"];

export function ConstantStore(context: DevModeContext): ConstantStoreType {
  const runtimeChain = context.pjsApi.runtimeChain.toUpperCase();
  const runtime = runtimeChain
    .split(" ")
    .filter((v) => Object.keys(RUNTIME_CONSTANTS).includes(v))
    .join();
  return RUNTIME_CONSTANTS[runtime];
}
