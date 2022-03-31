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

use {
	super::*,
	frame_support::pallet_prelude::*,
	frame_support::{ensure, traits::Get, StorageDoubleMap, StorageMap},
	sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
};

pub mod candidates;
mod common;
// It is important to automatically claim rewards before updating
// the amount of shares since pending rewards are stored per share.
pub mod auto_compounding;
pub mod leaving;
pub mod manual_claim;
