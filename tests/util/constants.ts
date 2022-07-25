export const SPECS_PATH = `./moonbeam-test-specs`;

export const DEBUG_MODE = process.env.DEBUG_MODE || false;
export const DISPLAY_LOG = process.env.MOONBEAM_LOG || false;
export const MOONBEAM_LOG = process.env.MOONBEAM_LOG || "info";

export const BINARY_PATH = process.env.BINARY_PATH || `../target/release/moonbeam`;
export const RELAY_BINARY_PATH = process.env.RELAY_BINARY_PATH || `../target/release/polkadot`;
export const RELAY_LOG = process.env.RELAY_LOG;

// Is undefined by default as the path is dependent of the runtime.
export const OVERRIDE_RUNTIME_PATH = process.env["OVERRIDE_RUNTIME_PATH"] || undefined;
export const SPAWNING_TIME = 20000;
export const ETHAPI_CMD = process.env.ETHAPI_CMD || "";
export const WASM_RUNTIME_OVERRIDES = process.env.WASM_RUNTIME_OVERRIDES || "";

export const RELAY_CHAIN_NODE_NAMES = ["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie", "One"];

// Test variables
export const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";
export const TREASURY_ACCOUNT = "0x6d6f646c70632f74727372790000000000000000";
export const GLMR = 1_000_000_000_000_000_000n;
export const MILLIGLMR = 1_000_000_000_000_000n;
export const MICROGLMR = 1_000_000_000_000n;
export const DEFAULT_GENESIS_BALANCE = 2n ** 80n;
export const DEFAULT_GENESIS_STAKING = 1_000n * GLMR;
export const DEFAULT_GENESIS_MAPPING = 100n * GLMR;
export const PROPOSAL_AMOUNT = 1000n * GLMR;
export const VOTE_AMOUNT = 10n * GLMR;
export const MIN_GLMR_STAKING = 1000n * GLMR;
export const MIN_GLMR_DELEGATOR = 1n * GLMR;

// Current gas per second
export const GAS_PER_SECOND = 40_000_000;
// The real computation is 1_000_000_000_000 / 40_000_000, but we simplify to avoid bigint.
export const GAS_PER_WEIGHT = 1_000_000 / 40;

// Our weight limit is 500ms.
export const BLOCK_TX_LIMIT = GAS_PER_SECOND * 0.5;

// Current implementation is limiting block transactions to 75% of the block gas limit
export const BLOCK_TX_GAS_LIMIT = BLOCK_TX_LIMIT * 0.75;
// 85_800_000 Weight per extrinsics
export const EXTRINSIC_BASE_COST = 85_800_000 / GAS_PER_WEIGHT;

// Maximum extrinsic weight is taken from the max allowed transaction weight per block,
// minus the block initialization (10%) and minus the extrinsic base cost.
export const EXTRINSIC_GAS_LIMIT = BLOCK_TX_GAS_LIMIT - BLOCK_TX_LIMIT * 0.1 - EXTRINSIC_BASE_COST;

// Weight per gas mapping
export const WEIGHT_PER_GAS = 1_000_000_000_000n / 40_000_000n;

export const MIN_GAS_PRICE = 1_000_000_000n;

export const PRECOMPILE_PARACHAIN_STAKING_ADDRESS = "0x0000000000000000000000000000000000000800";
export const PRECOMPILE_CROWDLOAN_REWARDS_ADDRESS = "0x0000000000000000000000000000000000000801";
export const PRECOMPILE_NATIVE_ERC20_ADDRESS = "0x0000000000000000000000000000000000000802";
export const PRECOMPILE_DEMOCRACY_ADDRESS = "0x0000000000000000000000000000000000000803";
export const PRECOMPILE_XTOKENS_ADDRESS = "0x0000000000000000000000000000000000000804";
export const PRECOMPILE_RELAY_ENCODER_ADDRESS = "0x0000000000000000000000000000000000000805";
export const PRECOMPILE_XCM_TRANSACTOR_ADDRESS = "0x0000000000000000000000000000000000000806";
export const PRECOMPILE_AUTHOR_MAPPING_ADDRESS = "0x0000000000000000000000000000000000000807";
export const PRECOMPILE_BATCH_ADDRESS = "0x0000000000000000000000000000000000000808";
export const PRECOMPILE_RANDOMNESS_ADDRESS = "0x0000000000000000000000000000000000000809";

export const MINUTES = 60 / 12;
export const HOURS = MINUTES * 60;
export const DAYS = HOURS * 24;

export const CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS = 0;
export const CONTRACT_RANDOMNESS_STATUS_PENDING = 1;
export const CONTRACT_RANDOMNESS_STATUS_READY = 2;
export const CONTRACT_RANDOMNESS_STATUS_EXPIRED = 3;
