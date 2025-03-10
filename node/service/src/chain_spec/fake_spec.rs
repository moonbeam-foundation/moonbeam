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

#![allow(clippy::todo)]

/// Fake specifications, only a workaround to compile with runtime optional features.
/// It's a zero variant enum to ensure at compile time that we never instantiate this type.
pub enum FakeSpec {}

impl FakeSpec {
	/// Parse json file into a `ChainSpec`
	pub fn from_json_file(_path: std::path::PathBuf) -> Result<Self, String> {
		unimplemented!()
	}
}

impl sp_runtime::BuildStorage for FakeSpec {
	fn assimilate_storage(&self, _storage: &mut sp_core::storage::Storage) -> Result<(), String> {
		todo!()
	}
}

impl sc_service::ChainSpec for FakeSpec {
	fn name(&self) -> &str {
		todo!()
	}

	fn id(&self) -> &str {
		todo!()
	}

	fn chain_type(&self) -> sc_chain_spec::ChainType {
		todo!()
	}

	fn boot_nodes(&self) -> &[sc_network::config::MultiaddrWithPeerId] {
		todo!()
	}

	fn telemetry_endpoints(&self) -> &Option<sc_telemetry::TelemetryEndpoints> {
		todo!()
	}

	fn protocol_id(&self) -> Option<&str> {
		todo!()
	}

	fn fork_id(&self) -> Option<&str> {
		todo!()
	}

	fn properties(&self) -> sc_chain_spec::Properties {
		todo!()
	}

	fn extensions(&self) -> &dyn sc_chain_spec::GetExtension {
		todo!()
	}

	fn extensions_mut(&mut self) -> &mut dyn sc_chain_spec::GetExtension {
		todo!()
	}

	fn add_boot_node(&mut self, _addr: sc_network::config::MultiaddrWithPeerId) {
		todo!()
	}

	fn as_json(&self, _raw: bool) -> Result<String, String> {
		todo!()
	}

	fn as_storage_builder(&self) -> &dyn sp_runtime::BuildStorage {
		todo!()
	}

	fn cloned_box(&self) -> Box<dyn polkadot_service::ChainSpec> {
		todo!()
	}

	fn set_storage(&mut self, _storage: sp_runtime::Storage) {
		todo!()
	}

	fn code_substitutes(&self) -> std::collections::BTreeMap<String, Vec<u8>> {
		todo!()
	}
}
