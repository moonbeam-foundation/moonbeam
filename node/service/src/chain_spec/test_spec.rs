// Copyright 2019-2025 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

//! Embedded specs for testing purposes, must be compiled with --features=test-spec
use crate::chain_spec::moonbase::ChainSpec;
use crate::chain_spec::Extensions;
use crate::HostFunctions;
use cumulus_primitives_core::ParaId;
use sc_service::ChainType;

/// Generate testing chain_spec for staking integration tests with accounts initialized for
/// collating and nominating.
pub fn staking_spec(para_id: ParaId) -> ChainSpec {
	ChainSpec::builder(
		moonbase_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!"),
		Extensions {
			relay_chain: "westend_local".into(),
			para_id: para_id.into(),
		},
	)
	.with_name("Moonbase Development Testnet")
	.with_id("staking")
	.with_chain_type(ChainType::Local)
	.with_properties(
		serde_json::from_str("{\"tokenDecimals\": 18}").expect("Provided valid json map"),
	)
	.with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
	.build()
}

#[cfg(feature = "lazy-loading")]
pub fn lazy_loading_spec_builder() -> sc_chain_spec::ChainSpecBuilder<Extensions, HostFunctions> {
	crate::chain_spec::moonbeam::ChainSpec::builder(
		moonbeam_runtime::WASM_BINARY.expect("WASM binary was not build, please build it!"),
		Default::default(),
	)
	.with_name("Lazy Loading")
	.with_id("lazy_loading")
	.with_chain_type(ChainType::Development)
	.with_properties(
		serde_json::from_str(
			"{\"tokenDecimals\": 18, \"tokenSymbol\": \"GLMR\", \"SS58Prefix\": 1284}",
		)
		.expect("Provided valid json map"),
	)
	.with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
}
