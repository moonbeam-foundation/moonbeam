// Copyright 2024 Moonbeam foundation
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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_xcm_weight_trader
pub trait WeightInfo {
    fn add_asset() -> Weight;
    fn edit_asset() -> Weight;
    fn pause_asset_support() -> Weight;
    fn resume_asset_support() -> Weight;
    fn remove_asset() -> Weight;
}

// For tests only
impl WeightInfo for () {
    fn add_asset() -> Weight {
        Weight::default()
    }
    fn edit_asset() -> Weight {
        Weight::default()
    }
    fn pause_asset_support() -> Weight {
        Weight::default()
    }
    fn resume_asset_support() -> Weight {
        Weight::default()
    }
    fn remove_asset() -> Weight {
        Weight::default()
    }
}