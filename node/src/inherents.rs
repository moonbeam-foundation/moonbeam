// Copyright 2019-2021 PureStake Inc.
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

//! This module is responsible for building the inherent data providers that a Moonbeam node needs.
//!
//! A service builder can call the `build_inherent_data_providers` function to get the providers
//! the node needs based on the parameters it passed in.
//!
//! This module also includes a MOCK inherent data provider for the validataion
//! data inherent. This mock provider provides stub data that does not represent anything "real"
//! about the external world, but can pass the runtime's checks. This is useful in testing
//! for example, running the --dev service without a relay chain backbone.

use cumulus_primitives_core::PersistedValidationData;
use cumulus_primitives_parachain_inherent::{ParachainInherentData, INHERENT_IDENTIFIER};
use parity_scale_codec::Encode;
use sp_core::H160;
use sp_inherents::{InherentData, InherentDataProviders, InherentIdentifier, ProvideInherentData};
use sp_timestamp::InherentError;

use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;

/// Build the inherent data providers for the node.
///
/// Not all nodes will need all inherent data providers:
/// - The author provider is only necessary for block producing nodes
/// - The validation data provider can be mocked.
pub fn build_inherent_data_providers(
	author: Option<H160>,
	mock: bool,
) -> Result<InherentDataProviders, sc_service::Error> {
	let providers = InherentDataProviders::new();

	// Timestamp provider. Needed in all nodes.
	providers
		.register_provider(sp_timestamp::InherentDataProvider)
		.map_err(Into::into)
		.map_err(sp_consensus::error::Error::InherentData)?;

	// Author ID Provider for authoring node only.
	if let Some(account) = author {
		providers
			.register_provider(author_inherent::InherentDataProvider(account.encode()))
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	}

	// Parachain inherent provider, only for dev-service nodes.
	if mock {
		providers
			.register_provider(MockValidationDataInherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	}

	// When we are not mocking the validation data, we do not register a real validation data
	// provider here. The validation data inherent is inserted manually by the cumulus colaltor
	// https://github.com/paritytech/cumulus/blob/c3e3f443/collator/src/lib.rs#L274-L321

	Ok(providers)
}

/// Inherent data provider that supplies mocked validation data.
///
/// This is useful when running a node that is not actually backed by any relay chain.
/// For example when running a local node, or running integration tests.
struct MockValidationDataInherentDataProvider;

impl ProvideInherentData for MockValidationDataInherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&INHERENT_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		// Use the "sproof" (spoof proof) builder to build valid mock state root and proof.
		let (relay_storage_root, proof) =
			RelayStateSproofBuilder::default().into_state_root_and_proof();

		let data = ParachainInherentData {
			validation_data: PersistedValidationData {
				parent_head: Default::default(),
				relay_parent_storage_root: relay_storage_root,
				relay_parent_number: Default::default(),
				max_pov_size: Default::default(),
			},
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
			relay_chain_state: proof,
		};

		inherent_data.put_data(INHERENT_IDENTIFIER, &data)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&INHERENT_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}
