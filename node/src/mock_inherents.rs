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
	para_id: u64,//todo type?
}

impl ProvideInherentData for MockValidationDataInherentDataProvider {
	fn inherent_identifier(&self) -> &'static InherentIdentifier {
		&VALIDATION_IDENTIFIER
	}

	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		todo!("what data do I actually need to mock here? Into the implementors guide we go")

		// polkadot/runtime/src/parachains/inclusion_inherent.rs
		// polkadot/primitives/src/v1.rs

		// TIMESTAMP.with(|x| {
		// 	*x.borrow_mut() += SLOT_DURATION;
		// 	inherent_data.put_data(TIMESTAMP_IDENTIFIER, &*x.borrow())
		// })
	}

	fn error_to_string(&self, error: &[u8]) -> Option<String> {
		InherentError::try_from(&VALIDATION_IDENTIFIER, error).map(|e| format!("{:?}", e))
	}
}
