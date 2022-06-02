export const SPECS_PATH = `./moonbeam-test-specs`;

export const DEBUG_MODE = process.env.DEBUG_MODE || false;
export const DISPLAY_LOG = process.env.MOONBEAM_LOG || false;
export const MOONBEAM_LOG = process.env.MOONBEAM_LOG || "info";

export const BINARY_PATH = process.env.BINARY_PATH || `../target/release/moonbeam`;
export const RELAY_BINARY_PATH = process.env.RELAY_BINARY_PATH || `../target/release/polkadot`;

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
export const MIN_GLMR_NOMINATOR = 5n * GLMR;

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

// for author mapping
export const ALITH_AUTHOR_ID = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
export const BALTATHAR_AUTHOR_ID =
  "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
export const CHARLETH_AUTHOR_ID =
  "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";

// Weight per gas mapping
export const WEIGHT_PER_GAS = 1_000_000_000_000n / 40_000_000n;

export const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

export const PRECOMPILE_AUTHOR_MAPPING_ADDRESS = "0x0000000000000000000000000000000000000807";
export const PRECOMPILE_NATIVE_ERC20_ADDRESS = "0x0000000000000000000000000000000000000802";
