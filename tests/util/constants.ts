export const SPECS_PATH = `./moonbeam-test-specs`;

export const DEBUG_MODE = process.env.DEBUG_MODE || false;
export const DISPLAY_LOG = process.env.MOONBEAM_LOG || false;
export const MOONBEAM_LOG = process.env.MOONBEAM_LOG || "info";

export const BINARY_PATH = process.env.BINARY_PATH || `../target/release/moonbeam`;
export const SPAWNING_TIME = 10000;

// Test variables
export const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
export const GENESIS_ACCOUNT_PRIVATE_KEY =
  "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
export const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
export const GLMR = 1_000_000_000_000_000_000n;
export const DEFAULT_GENESIS_BALANCE = 2n ** 80n;
export const DEFAULT_GENESIS_STAKING = 1_000n * GLMR;
export const DEFAULT_GENESIS_MAPPING = 100n * GLMR;
export const GENESIS_ACCOUNT_BALANCE =
  DEFAULT_GENESIS_BALANCE;

// This is Alice
export const COLLATOR_ACCOUNT = "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac";
export const COLLATOR_ACCOUNT_BALANCE =
  DEFAULT_GENESIS_BALANCE - DEFAULT_GENESIS_STAKING - DEFAULT_GENESIS_MAPPING;

// Current gas per second
export const GAS_PER_SECOND = 40_000_000;
// The real computation is 1_000_000_000_000 / 40_000_000, but we simplify to avoid bigint.
export const GAS_PER_WEIGHT = 1_000_000 / 40;

// Our weight limit is 500ms.
export const BLOCK_TX_LIMIT = GAS_PER_SECOND * 0.5;

// Current implementation is limiting block transactions to 75% of the block gas limit
export const BLOCK_TX_GAS_LIMIT = BLOCK_TX_LIMIT * 0.75;
// 125_000_000 Weight per extrinsics
export const EXTRINSIC_BASE_COST = 125_000_000 / GAS_PER_WEIGHT;

// Maximum extrinsic weight is taken from the max allowed transaction weight per block,
// minus the block initialization (10%) and minus the extrinsic base cost.
export const EXTRINSIC_GAS_LIMIT = BLOCK_TX_GAS_LIMIT - BLOCK_TX_LIMIT * 0.1 - EXTRINSIC_BASE_COST;
