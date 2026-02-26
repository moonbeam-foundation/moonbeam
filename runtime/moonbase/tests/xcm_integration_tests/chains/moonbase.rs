// Copyright 2019-2025 Moonbeam Foundation.
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

//! Moonbase chain setup using the real runtime.

use crate::common::ExtBuilder;
use moonbase_runtime::{currency::UNIT, AccountId};
use sp_io::TestExternalities;

/// Moonbase's parachain ID
pub const MOONBASE_PARA_ID: u32 = 1000;

/// Create test externalities for Moonbase
pub fn moonbase_ext() -> TestExternalities {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from([1u8; 20]), UNIT * 1000),
			(AccountId::from([2u8; 20]), UNIT * 1000),
			(AccountId::from([3u8; 20]), UNIT * 1000),
		])
		.build()
}
