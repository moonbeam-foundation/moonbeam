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

//! # Migrations

pub mod v1 {
	use crate::{
		Config as ParachainStakingConfig, Pallet as ParachainStakingPallet, Round, RoundIndex,
		RoundInfo,
	};
	use cumulus_pallet_parachain_system::{
		consensus_hook::UnincludedSegmentCapacity, ConsensusHook, RelayChainStateProof,
	};
	use frame_support::{
		pallet_prelude::Weight,
		sp_runtime::traits::{Block, Header},
		traits::{Get, StorageVersion},
	};
	use frame_system::pallet_prelude::BlockNumberFor;
	use parity_scale_codec::{Decode, Encode};

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode)]
	pub struct OldRoundInfo<BlockNumber> {
		pub current: RoundIndex,
		pub first: BlockNumber,
		pub length: u32,
	}

	pub struct ConsensusHookWrapperForMigration<Inner, T>(core::marker::PhantomData<(Inner, T)>);
	impl<Inner, T> ConsensusHook for ConsensusHookWrapperForMigration<Inner, T>
	where
		Inner: ConsensusHook,
		T: ParachainStakingConfig,
		T: frame_system::Config,
		u32: From<<<<T as frame_system::Config>::Block as Block>::Header as Header>::Number>,
	{
		fn on_state_proof(
			state_proof: &RelayChainStateProof,
		) -> (Weight, UnincludedSegmentCapacity) {
			let mut wrapper_weight = Weight::zero();

			if StorageVersion::get::<ParachainStakingPallet<T>>() == 0 {
				let _ = Round::<T>::translate::<OldRoundInfo<BlockNumberFor<T>>, _>(|v0| {
					// Fetch the old round info values
					let old_current = v0
						.expect("old current round value must be present!")
						.current;
					let old_first = v0.expect("old first round value must be present!").first;
					let old_length = v0.expect("old round length value must be present!").length;

					// Fetch the last parachain block
					let para_block: u32 = frame_system::Pallet::<T>::block_number().into();

					// Calculate how many blocks have passed so far in the current round
					let para_block_diff = para_block.saturating_sub(old_first.into());

					// Calculate the percentage of the round so far (before the migration)
					let percentage = (para_block_diff)
						.saturating_mul(100)
						.saturating_div(old_length);

					// Calculate how many blocks should we substract from the relay slot number
					// given the new duration (round_info.length * 2) to have a first relay slot of the
					// round that corresponds with the percentage calculated in the step above.
					let new_block_diff: u64 = percentage
						.saturating_mul(old_length * 2)
						.saturating_div(100)
						.into();

					// Read the relay slot from the state proof
					let relay_slot = state_proof
						.read_slot()
						.expect("Relay slot must be included!");

					// Calculate the updated first block of the round
					let new_first_block = u64::from(relay_slot).saturating_sub(new_block_diff);

					Some(RoundInfo {
						current: old_current,
						first: new_first_block,
						length: old_length * 2,
					})
				});

				StorageVersion::new(1).put::<ParachainStakingPallet<T>>();

				wrapper_weight += T::DbWeight::get().reads_writes(2, 2);
			}
			let (weight, capacity) = Inner::on_state_proof(state_proof);

			(weight.saturating_add(wrapper_weight), capacity)
		}
	}
}
