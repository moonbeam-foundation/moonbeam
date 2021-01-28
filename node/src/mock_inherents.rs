use cumulus_primitives::{
	inherents::{ValidationDataType, VALIDATION_DATA_IDENTIFIER},
	PersistedValidationData, TransientValidationData, ValidationData,
};
use sp_core::H256;
/// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
/// Each call will increment timestamp by slot_duration making Aura think time has passed.
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

// This module introduces a FAKE timestamp inherent data provider which will
// always include timestamps that are one aura-slot-duration apart from one another.
// This allows us to use manual seal to author blocks quickly without violating
// timestamp assumptions made by either the Aura pallet or the Timestamp pallet.
//
// This code was taken from https://github.com/paritytech/frontier/pull/170
// When moonbeam updates to a Frontier version that includes that PR, we should re-evaluate
// whether it makes sense to keep this here.
use sp_inherents::{InherentData, InherentIdentifier, ProvideInherentData};
use sp_timestamp::InherentError;
use std::cell::RefCell;

use moonbeam_runtime::MINIMUM_PERIOD;
//TODO get this from cumulus
use crate::sproof::RelayStateSproofBuilder;

//TODO make this a field on the struct
const SLOT_DURATION: u64 = MINIMUM_PERIOD * 2;

/// Mocked timestamp inherent data provider.
/// Provides a fake duration starting at 0 in millisecond for timestamp inherent.
/// Each call will increment timestamp by slot_duration making the runtime think time has passed.
pub struct MockTimestampInherentDataProvider;

// todo, should I be importing this from somewhere rather than recreating it myself
pub const TIMESTAMP_IDENTIFIER: InherentIdentifier = *b"timstap0";

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
			*x.borrow_mut() += SLOT_DURATION;
			inherent_data.put_data(TIMESTAMP_IDENTIFIER, &*x.borrow())
		})
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&TIMESTAMP_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}

pub struct MockValidationDataInherentDataProvider {
	//para_id: u64,//todo type? // todo also, do I even need this?
}

impl ProvideInherentData for MockValidationDataInherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&VALIDATION_DATA_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		// Use the "sproof" (spoof proof) builder to build valid mock state root and proof.
		let (root, proof) = RelayStateSproofBuilder::default().into_state_root_and_proof();

		let data = ValidationDataType {
			validation_data: ValidationData {
				persisted: PersistedValidationData {
					/// The parent head-data.
					parent_head: Vec::new().into(),
					/// The relay-chain block number this is in the context of.
					block_number: 0,
					/// The relay-chain block storage root this is in the context of.
					relay_storage_root: root,
					/// The list of MQC heads for the inbound channels paired with the sender para ids. This
					/// vector is sorted ascending by the para id and doesn't contain multiple entries with the same
					/// sender.
					hrmp_mqc_heads: Vec::new(),
					/// The MQC head for the DMQ.
					///
					/// The DMQ MQC head will be used by the validation function to authorize the downward messages
					/// passed by the collator.
					dmq_mqc_head: H256::zero(),
					/// The maximum legal size of a POV block, in bytes.
					max_pov_size: u32::max_value(),
				},
				transient: TransientValidationData {
					/// The maximum code size permitted, in bytes.
					max_code_size: u32::max_value(),
					/// The maximum head-data size permitted, in bytes.
					max_head_data_size: u32::max_value(),
					/// The balance of the parachain at the moment of validation.
					balance: 0,
					/// Whether the parachain is allowed to upgrade its validation code.
					///
					/// This is `Some` if so, and contains the number of the minimum relay-chain
					/// height at which the upgrade will be applied, if an upgrade is signaled
					/// now.
					///
					/// A parachain should enact its side of the upgrade at the end of the first
					/// parablock executing in the context of a relay-chain block with at least this
					/// height. This may be equal to the current perceived relay-chain block height, in
					/// which case the code upgrade should be applied at the end of the signaling
					/// block.
					code_upgrade_allowed: None,
					/// The number of messages pending of the downward message queue.
					dmq_length: 0,
				},
			},
			relay_chain_state: proof,
		};

		inherent_data.put_data(VALIDATION_DATA_IDENTIFIER, &data)
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&VALIDATION_DATA_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}
