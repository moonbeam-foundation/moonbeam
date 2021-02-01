// Copyright 2019-2020 PureStake Inc.
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
//! This module also includes MOCK inherent data providers for both the timestamp and validataion
//! data inherents. These mock providers provide stub data that does not represent anything "real"
//! about the external world, but can pass the runtime's checks. This is useful in testing
//! for example, running the --dev service without a relay chain backbone, or authoring block
//! extremely quickly in testing scenarios.

use cumulus_primitives::{
	inherents::{ValidationDataType, VALIDATION_DATA_IDENTIFIER},
	PersistedValidationData, ValidationData,
};
use parity_scale_codec::Encode;
use sp_core::H160;
use sp_inherents::{InherentData, InherentDataProviders, InherentIdentifier, ProvideInherentData};
use sp_timestamp::{InherentError, INHERENT_IDENTIFIER as TIMESTAMP_IDENTIFIER};
use std::cell::RefCell;

use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use moonbeam_runtime::MINIMUM_PERIOD;

/// Build the inherent data providers for the node.
///
/// Not all nodes will need all inherent data providers:
/// - The author provider is only necessary for block producing nodes
/// - The timestamp provider can be mocked.
/// - The validation data provider can be mocked.
pub fn build_inherent_data_providers(
	author: Option<H160>,
	mock: bool,
) -> Result<InherentDataProviders, sc_service::Error> {
	let providers = InherentDataProviders::new();

	if let Some(account) = author {
		providers
			.register_provider(author_inherent::InherentDataProvider(account.encode()))
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	}

	if mock {
		providers
			.register_provider(MockTimestampInherentDataProvider {
				duration: MINIMUM_PERIOD * 2,
			})
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;

		providers
			.register_provider(MockValidationDataInherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;
	} else {
		providers
			.register_provider(sp_timestamp::InherentDataProvider)
			.map_err(Into::into)
			.map_err(sp_consensus::error::Error::InherentData)?;

		// When we are not mocking the validation data ,we do not register a real validation data
		// provider here. The validation data inherent is inserted manually by the cumulus colaltor
		// https://github.com/paritytech/cumulus/blob/c3e3f443/collator/src/lib.rs#L274-L321
	}

	Ok(providers)
}

/// Mocked timestamp inherent data provider.
/// Provides a fake duration starting at 0 in millisecond for timestamp inherent.
/// Each call will increment timestamp by slot_duration making the runtime think time has passed.
/// This code was inspired by https://github.com/paritytech/frontier/pull/170
struct MockTimestampInherentDataProvider {
	duration: u64,
}

thread_local!(static TIMESTAMP: RefCell<u64> = RefCell::new(0));

impl ProvideInherentData for MockTimestampInherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&TIMESTAMP_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		TIMESTAMP.with(|x| {
			*x.borrow_mut() += self.duration;
			inherent_data.put_data(TIMESTAMP_IDENTIFIER, &*x.borrow())
		})
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&TIMESTAMP_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}

/// Inherent data provider that supplies mocked validation data.
///
/// This is useful when running a node that is not actually backed by any relay chain.
/// For example when running a local node, or running integration tests.
struct MockValidationDataInherentDataProvider;

impl ProvideInherentData for MockValidationDataInherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&VALIDATION_DATA_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		// Use the "sproof" (spoof proof) builder to build valid mock state root and proof.
		let (relay_storage_root, proof) =
			RelayStateSproofBuilder::default().into_state_root_and_proof();

		let data = ValidationDataType {
			validation_data: ValidationData {
				persisted: PersistedValidationData {
					parent_head: Default::default(),
					block_number: Default::default(),
					relay_storage_root,
					hrmp_mqc_heads: Default::default(),
					dmq_mqc_head: Default::default(),
					max_pov_size: Default::default(),
				},
				transient: Default::default(),
			},
			relay_chain_state: proof,
		};

		inherent_data.put_data(VALIDATION_DATA_IDENTIFIER, &data)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&VALIDATION_DATA_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}
