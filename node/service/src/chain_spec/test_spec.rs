// Copyright 2019-2022 PureStake Inc.
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
use crate::chain_spec::moonbase::{testnet_genesis, ChainSpec};
use crate::chain_spec::{get_from_seed, Extensions};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use moonbase_runtime::{currency::UNIT, AccountId, WASM_BINARY};
use nimbus_primitives::NimbusId;
use sc_service::ChainType;

/// Generate testing chain_spec for staking integration tests with accounts initialized for
/// collating and nominating.
pub fn staking_spec(para_id: ParaId) -> ChainSpec {
	ChainSpec::builder(
		WASM_BINARY.expect("WASM binary was not build, please build it!"),
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
	.with_genesis_config(testnet_genesis(
		// Root
		AccountId::from(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
		// Treasury Council members: Baltathar, Charleth and Dorothy
		vec![
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
			AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")),
			AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")),
		],
		// Open Tech Committee members: Alith and Baltathar
		vec![
			AccountId::from(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
		],
		// Collators
		vec![
			(
				AccountId::from(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
				get_from_seed::<NimbusId>("Alice"),
				1_000 * UNIT,
			),
			(
				AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")),
				get_from_seed::<NimbusId>("Faith"),
				1_000 * UNIT,
			),
		],
		// Delegations
		vec![],
		// Endowed accounts (each minted 1 << 80 balance)
		vec![
			// Alith, Baltathar, Charleth, Dorothy and Faith
			AccountId::from(hex!("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b")),
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
			AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")),
			AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")),
			AccountId::from(hex!("C0F0f4ab324C46e55D02D0033343B4Be8A55532d")),
			// Additional accounts
			AccountId::from(hex!("Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB")),
			AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
		],
		3_000_000 * UNIT,
		para_id,
		// Chain ID
		1280,
	))
	.build()
}
