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

//! Migrations
use crate::{Config, CurrentVrfInput};
use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;
use session_keys_primitives::VrfInput;

/// Set initial CurrentVrfInput value to VrfInput::default() for the first block
pub struct InitializeVrfInput<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for InitializeVrfInput<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "InitializeVrfInput", "running migration");

		CurrentVrfInput::<T>::put(VrfInput::default());

		T::DbWeight::get().write
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		// trivial migration
		Ok(())
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		// trivial migration
		Ok(())
	}
}
