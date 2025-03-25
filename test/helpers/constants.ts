// constants.ts -  Any common values here should be moved to moonwall if suitable

import type { GenericContext } from "@moonwall/cli";
import {
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_GENESIS_LOCK_BALANCE,
  ALITH_GENESIS_RESERVE_BALANCE,
} from "@moonwall/util";

const KILOWEI = 1_000n;

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

const MOONBASE_CONSTANTS = {
  SUPPLY_FACTOR: 1n,
};

const MOONRIVER_CONSTANTS = {
  SUPPLY_FACTOR: 1n,
};

const MOONBEAM_CONSTANTS = {
  SUPPLY_FACTOR: 100n,
};

// Fees and gas limits
export const RUNTIME_CONSTANTS = {
  MOONBASE: {
    ...MOONBASE_CONSTANTS,
    GENESIS_FEE_MULTIPLIER: 8_000_000_000_000_000_000n,
    MIN_FEE_MULTIPLIER: 100_000_000_000_000_000n,
    MAX_FEE_MULTIPLIER: 100_000_000_000_000_000_000_000n,
    WEIGHT_FEE: new RuntimeConstant({
      3400: (50n * KILOWEI * MOONBASE_CONSTANTS.SUPPLY_FACTOR) / 4n,
      0: 50n * KILOWEI * MOONBASE_CONSTANTS.SUPPLY_FACTOR,
    }),

    GENESIS_BASE_FEE: new RuntimeConstant({ 3400: 2_500_000_000n, 0: 10_000_000_000n }),
    // (MinimumMultiplier = 0.1) * WEIGHT_FEE * WEIGHT_PER_GAS
    MIN_BASE_FEE: new RuntimeConstant({ 3400: 31_250_000n, 0: 125_000_000n }),
    // (MaximumMultiplier = 100_000) * WEIGHT_FEE * WEIGHT_PER_GAS
    MAX_BASE_FEE: new RuntimeConstant({ 3400: 3_125_000_000_000n, 0: 12_500_000_000_000n }),

    TARGET_FILL_PERMILL: new RuntimeConstant({ 3000: 350_000n, 2801: 500_000n, 0: 250_000n }),
    // Deadline for block production in milliseconds
    DEADLINE_MILISECONDS: new RuntimeConstant({ 2800: 2000n, 0: 500n }),
    // 2 seconds of weight
    BLOCK_WEIGHT_LIMIT: new RuntimeConstant({ 2900: 2_000_000_000_000n, 0: 500_000_000_000n }),
    // Gas limit considering the block utilization threshold (75%)
    GAS_LIMIT: new RuntimeConstant({ 2900: 60_000_000n, 0: 15_000_000n }),
    // Maximum extrinsic weight is taken from the max allowed transaction weight per block (75%),
    // minus the block initialization (10%) and minus the extrinsic base cost.
    EXTRINSIC_GAS_LIMIT: new RuntimeConstant({ 2900: 52_000_000n, 0: 13_000_000n }),
    // Maximum Gas to PoV ratio used in the gasometer
    GAS_PER_POV_BYTES: new RuntimeConstant({ 3600: 8n, 2900: 16n, 0: 4n }),
    // Maximum PoV per block
    MAX_POV: new RuntimeConstant({ 3600: 7_500_000n, 0: 3_750_000n }),
    // Storage read/write costs
    STORAGE_READ_COST: 41_742_000n,
    // Weight to gas conversion ratio
    WEIGHT_TO_GAS_RATIO: 25_000n,
  },
  MOONRIVER: {
    ...MOONRIVER_CONSTANTS,
    GENESIS_FEE_MULTIPLIER: 10_000_000_000_000_000_000n,
    MIN_FEE_MULTIPLIER: 1_000_000_000_000_000_000n,
    MAX_FEE_MULTIPLIER: 100_000_000_000_000_000_000_000n,
    WEIGHT_FEE: new RuntimeConstant({
      3400: (50n * KILOWEI * MOONRIVER_CONSTANTS.SUPPLY_FACTOR) / 4n,
      0: 50n * KILOWEI * MOONRIVER_CONSTANTS.SUPPLY_FACTOR,
    }),

    GENESIS_BASE_FEE: new RuntimeConstant({ 3400: 3_125_000_000n, 0: 12_500_000_000n }),
    // (MinimumMultiplier = 1) * WEIGHT_FEE * WEIGHT_PER_GAS
    MIN_BASE_FEE: new RuntimeConstant({ 3400: 312_500_000n, 0: 1_250_000_000n }),
    // (MaximumMultiplier = 100_000) * WEIGHT_FEE * WEIGHT_PER_GAS
    MAX_BASE_FEE: new RuntimeConstant({ 3400: 31_250_000_000_000n, 0: 125_000_000_000_000n }),

    TARGET_FILL_PERMILL: new RuntimeConstant({ 3000: 350_000n, 2801: 500_000n, 0: 250_000n }),
    // Deadline for block production in milliseconds
    DEADLINE_MILISECONDS: new RuntimeConstant({ 3000: 2000n, 0: 500n }),
    // Calculated as the weight per second by the deadline in seconds
    BLOCK_WEIGHT_LIMIT: new RuntimeConstant({
      3100: 2_000_000_000_000n,
      3000: 1_000_000_000_000n,
      0: 500_000_000_000n,
    }),
    // Gas limit considering the block utilization threshold (75%)
    GAS_LIMIT: new RuntimeConstant({ 3100: 60_000_000n, 3000: 30_000_000n, 0: 15_000_000n }),
    // Maximum extrinsic weight is taken from the max allowed transaction weight per block (75%),
    // minus the block initialization (10%) and minus the extrinsic base cost.
    EXTRINSIC_GAS_LIMIT: new RuntimeConstant({
      3100: 52_000_000n,
      3000: 26_000_000n,
      0: 13_000_000n,
    }),
    // Maximum Gas to PoV ratio used in the gasometer
    GAS_PER_POV_BYTES: new RuntimeConstant({ 3100: 16n, 3000: 8n, 0: 4n }),
    // Maximum PoV per block
    MAX_POV: new RuntimeConstant({ 0: 3_750_000n }),
  },
  MOONBEAM: {
    ...MOONBEAM_CONSTANTS,
    GENESIS_FEE_MULTIPLIER: 8_000_000_000_000_000_000n,
    MIN_FEE_MULTIPLIER: 1_000_000_000_000_000_000n,
    MAX_FEE_MULTIPLIER: 100_000_000_000_000_000_000_000n,
    WEIGHT_FEE: new RuntimeConstant({
      3400: (50n * KILOWEI * MOONBEAM_CONSTANTS.SUPPLY_FACTOR) / 4n,
      0: 50n * KILOWEI * MOONBEAM_CONSTANTS.SUPPLY_FACTOR,
    }),

    GENESIS_BASE_FEE: new RuntimeConstant({ 3400: 250_000_000_000n, 0: 1_000_000_000_000n }),
    // (MinimumMultiplier = 1) * WEIGHT_FEE * WEIGHT_PER_GAS
    MIN_BASE_FEE: new RuntimeConstant({ 3400: 31_250_000_000n, 0: 125_000_000_000n }),
    // (MaximumMultiplier = 100_000) * WEIGHT_FEE * WEIGHT_PER_GAS
    MAX_BASE_FEE: new RuntimeConstant({ 3400: 3_125_000_000_000_000n, 0: 12_500_000_000_000_000n }),

    TARGET_FILL_PERMILL: new RuntimeConstant({ 3000: 350_000n, 2801: 500_000n, 0: 250_000n }),
    // Deadline for block production in milliseconds
    DEADLINE_MILISECONDS: new RuntimeConstant({ 3000: 2000n, 0: 500n }),
    // Calculated as the weight per second by the deadline in seconds
    BLOCK_WEIGHT_LIMIT: new RuntimeConstant({
      3200: 2_000_000_000_000n,
      3100: 1_000_000_000_000n,
      0: 500_000_000_000n,
    }),
    // Gas limit considering the block utilization threshold (75%)
    GAS_LIMIT: new RuntimeConstant({ 3200: 60_000_000n, 3100: 30_000_000n, 0: 15_000_000n }),
    // Maximum extrinsic weight is taken from the max allowed transaction weight per block (75%),
    // minus the block initialization (10%) and minus the extrinsic base cost.
    EXTRINSIC_GAS_LIMIT: new RuntimeConstant({
      3200: 52_000_000n,
      3100: 26_000_000n,
      0: 13_000_000n,
    }),
    // Maximum Gas to PoV ratio used in the gasometer
    GAS_PER_POV_BYTES: new RuntimeConstant({ 3200: 16n, 3100: 8n, 0: 4n }),
    // Maximum PoV per block
    MAX_POV: new RuntimeConstant({ 0: 3_750_000n }),
  },
} as const;

type ConstantStoreType = (typeof RUNTIME_CONSTANTS)["MOONBASE"];

export function ConstantStore(context: GenericContext): ConstantStoreType {
  const runtime = context.polkadotJs().consts.system.version.specName.toUpperCase();
  return RUNTIME_CONSTANTS[runtime];
}
