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

use codec::{Codec, Decode, Encode};
use fp_self_contained::IsSelfContained;
use frame_support::{
	dispatch::PostDispatchInfo,
	traits::{
		EnsureInherentsAreFirst, ExecuteBlock, OffchainWorker, OnFinalize, OnIdle, OnInitialize,
		OnRuntimeUpgrade,
	},
	unsigned::TransactionValidityError,
	weights::{DispatchInfo, GetDispatchInfo},
};
use sp_runtime::{
	traits::{self, Applyable, Checkable, Dispatchable, Header, NumberFor, ValidateUnsigned},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_std::prelude::*;

pub type CheckedOf<E, C> = <E as Checkable<C>>::Checked;
pub type CallOf<E, C> = <CheckedOf<E, C> as Applyable>::Call;
pub type OriginOf<E, C> = <CallOf<E, C> as Dispatchable>::Origin;

pub struct MoonbeamExecutive<
	System,
	Block,
	Context,
	UnsignedValidator,
	AllPallets,
	OnRuntimeUpgrade = (),
>(
	frame_executive::Executive<
		System,
		Block,
		Context,
		UnsignedValidator,
		AllPallets,
		OnRuntimeUpgrade,
	>,
);

impl<
		System: frame_system::Config + EnsureInherentsAreFirst<Block>,
		Block: traits::Block<Header = System::Header, Hash = System::Hash>,
		Context: Default,
		UnsignedValidator,
		AllPallets: OnRuntimeUpgrade
			+ OnInitialize<System::BlockNumber>
			+ OnIdle<System::BlockNumber>
			+ OnFinalize<System::BlockNumber>
			+ OffchainWorker<System::BlockNumber>,
		COnRuntimeUpgrade: OnRuntimeUpgrade,
	> ExecuteBlock<Block>
	for MoonbeamExecutive<System, Block, Context, UnsignedValidator, AllPallets, COnRuntimeUpgrade>
where
	Block::Extrinsic: Checkable<Context> + Codec + IsSelfContained,
	CheckedOf<Block::Extrinsic, Context>: Applyable + Codec + GetDispatchInfo,
	CallOf<Block::Extrinsic, Context>:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	OriginOf<Block::Extrinsic, Context>: From<Option<System::AccountId>>,
	UnsignedValidator: ValidateUnsigned<Call = CallOf<Block::Extrinsic, Context>>,
{
	fn execute_block(block: Block) {
		Self::execute_block_with_para_check(block, 4)
	}
}

impl<
		System: frame_system::Config + EnsureInherentsAreFirst<Block>,
		Block: traits::Block<Header = System::Header, Hash = System::Hash>,
		Context: Default,
		UnsignedValidator,
		AllPallets: OnRuntimeUpgrade
			+ OnInitialize<System::BlockNumber>
			+ OnIdle<System::BlockNumber>
			+ OnFinalize<System::BlockNumber>
			+ OffchainWorker<System::BlockNumber>,
		COnRuntimeUpgrade: OnRuntimeUpgrade,
	> MoonbeamExecutive<System, Block, Context, UnsignedValidator, AllPallets, COnRuntimeUpgrade>
where
	Block::Extrinsic: Checkable<Context> + Codec,
	CheckedOf<Block::Extrinsic, Context>: Applyable + GetDispatchInfo,
	CallOf<Block::Extrinsic, Context>:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	OriginOf<Block::Extrinsic, Context>: From<Option<System::AccountId>>,
	UnsignedValidator: ValidateUnsigned<Call = CallOf<Block::Extrinsic, Context>>,
{
	/// Apply extrinsic outside of the block execution function.
	///
	/// This doesn't attempt to validate anything regarding the block, but it builds a list of uxt
	/// hashes.
	pub fn apply_extrinsic(uxt: Block::Extrinsic) -> ApplyExtrinsicResult {
		frame_executive::Executive::<
			System,
			Block,
			Context,
			UnsignedValidator,
			AllPallets,
			COnRuntimeUpgrade,
		>::apply_extrinsic(uxt)
	}

	/// Start the execution of a particular block.
	pub fn initialize_block(header: &System::Header) {
		frame_executive::Executive::<
			System,
			Block,
			Context,
			UnsignedValidator,
			AllPallets,
			COnRuntimeUpgrade,
		>::initialize_block(header)
	}

	/// Finalize the block - it is up the caller to ensure that all header fields are valid
	/// except state-root.
	pub fn finalize_block() -> System::Header {
		frame_executive::Executive::<
			System,
			Block,
			Context,
			UnsignedValidator,
			AllPallets,
			COnRuntimeUpgrade,
		>::finalize_block()
	}

	/// Start an offchain worker and generate extrinsics.
	pub fn offchain_worker(header: &System::Header) {
		frame_executive::Executive::<
			System,
			Block,
			Context,
			UnsignedValidator,
			AllPallets,
			COnRuntimeUpgrade,
		>::offchain_worker(header)
	}

	pub fn validate_transaction(
		source: TransactionSource,
		uxt: Block::Extrinsic,
		block_hash: Block::Hash,
	) -> TransactionValidity {
		frame_executive::Executive::<
			System,
			Block,
			Context,
			UnsignedValidator,
			AllPallets,
			COnRuntimeUpgrade,
		>::validate_transaction(source, uxt, block_hash)
	}
}

impl<
		System: frame_system::Config + EnsureInherentsAreFirst<Block>,
		Block: traits::Block<Header = System::Header, Hash = System::Hash>,
		Context: Default,
		UnsignedValidator,
		AllPallets: OnRuntimeUpgrade
			+ OnInitialize<System::BlockNumber>
			+ OnIdle<System::BlockNumber>
			+ OnFinalize<System::BlockNumber>
			+ OffchainWorker<System::BlockNumber>,
		COnRuntimeUpgrade: OnRuntimeUpgrade,
	> MoonbeamExecutive<System, Block, Context, UnsignedValidator, AllPallets, COnRuntimeUpgrade>
where
	Block::Extrinsic: Checkable<Context> + Codec + IsSelfContained,
	CheckedOf<Block::Extrinsic, Context>: Applyable + Codec + GetDispatchInfo,
	CallOf<Block::Extrinsic, Context>:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	OriginOf<Block::Extrinsic, Context>: From<Option<System::AccountId>>,
	UnsignedValidator: ValidateUnsigned<Call = CallOf<Block::Extrinsic, Context>>,
{
	/// Actually execute all transitions for `block`.
	/// n is the number of different tasks spawned for signatures verification
	pub fn execute_block_with_para_check(block: Block, n: usize) {
		sp_io::init_tracing();
		sp_tracing::within_span! {
			sp_tracing::info_span!("execute_block", ?block);

			frame_executive::Executive::<
				System,
				Block,
				Context,
				UnsignedValidator,
				AllPallets,
				COnRuntimeUpgrade,
			>::initialize_block(block.header());

			// any initial checks
			frame_executive::Executive::<
				System,
				Block,
				Context,
				UnsignedValidator,
				AllPallets,
				COnRuntimeUpgrade,
			>::initial_checks(&block);

			let (header, extrinsics) = block.deconstruct();
			let encoded_extrinsics = extrinsics.iter().map(|e| e.encode()).collect();

			// Verify signatures
			let checked_extrinsics = match Self::check_parallel(extrinsics, n) {
				Ok(checked_extrinsics) => checked_extrinsics,
				Err(e) => {
					let err: &'static str = e.into();
					panic!("{}", err)
				}
			};

			// execute extrinsics
			Self::execute_checked_extrinsics_with_book_keeping(encoded_extrinsics, checked_extrinsics, *header.number());

			// any final checks
			frame_executive::Executive::<
				System,
				Block,
				Context,
				UnsignedValidator,
				AllPallets,
				COnRuntimeUpgrade,
			>::final_checks(&header);
		}
	}

	fn check_parallel(
		extrinsics: Vec<Block::Extrinsic>,
		n: usize,
	) -> Result<Vec<CheckedOf<Block::Extrinsic, Context>>, TransactionValidityError> {
		fn spawn_verify<
			Context: Default,
			Extrinsic: Checkable<Context, Checked = CheckedExtrinsic> + Codec,
			CheckedExtrinsic: Encode,
		>(
			data: Vec<u8>,
		) -> Vec<u8> {
			let stream = &mut &data[..];
			let extrinsics = Vec::<Extrinsic>::decode(stream).expect("Failed to decode");

			let mut checked_extrinsics = Vec::with_capacity(extrinsics.len());
			for extrinsic in extrinsics {
				match extrinsic.check(&Default::default()) {
					Ok(checked_extrinsic) => checked_extrinsics.push(checked_extrinsic),
					Err(e) => {
						return Result::<Vec<CheckedExtrinsic>, TransactionValidityError>::Err(e)
							.encode()
					}
				}
			}
			Result::<Vec<CheckedExtrinsic>, TransactionValidityError>::Ok(checked_extrinsics)
				.encode()
		}

		if extrinsics.is_empty() {
			Ok(Vec::new())
		} else {
			let extrinsics_len = extrinsics.len();
			let mut substrate_extrinsics = Vec::with_capacity(extrinsics_len);
			let mut eth_extrinsics = Vec::with_capacity(extrinsics_len);
			for extrinsic in extrinsics {
				if extrinsic.is_self_contained() {
					eth_extrinsics.push(extrinsic);
				} else {
					substrate_extrinsics.push(extrinsic);
				}
			}

			// Spawn eth extrinsics check
			let handles: Vec<sp_tasks::DataJoinHandle> = if eth_extrinsics.is_empty() {
				Vec::new()
			} else {
				let chunk_size = core::cmp::max(eth_extrinsics.len() / n, 1);
				eth_extrinsics
					.chunks(chunk_size)
					.map(|chunk| {
						let mut async_payload = Vec::new();
						chunk.encode_to(&mut async_payload);
						sp_tasks::spawn(
							spawn_verify::<
								Context,
								Block::Extrinsic,
								CheckedOf<Block::Extrinsic, Context>,
							>,
							async_payload,
						)
					})
					.collect()
			};

			let mut all_checked_extrinsics = Vec::with_capacity(extrinsics_len);

			// Check substrate extrinsics in main task
			// (because CheckGenesis and CheckMortality need storage access)
			for substrate_extrinsic in substrate_extrinsics {
				all_checked_extrinsics.push(substrate_extrinsic.check(&Default::default())?);
			}

			for handle in handles {
				let checked_extrinsics = <Result<
					Vec<CheckedOf<Block::Extrinsic, Context>>,
					TransactionValidityError,
				> as Decode>::decode(&mut &handle.join()[..])
				.expect("Failed to decode result")?;
				all_checked_extrinsics.extend(checked_extrinsics);
			}

			Ok(all_checked_extrinsics)
		}
	}

	/// Execute given extrinsics and take care of post-extrinsics book-keeping.
	fn execute_checked_extrinsics_with_book_keeping(
		encoded_extrinsics: Vec<Vec<u8>>,
		extrinsics: Vec<CheckedOf<Block::Extrinsic, Context>>,
		block_number: NumberFor<Block>,
	) {
		extrinsics.into_iter().zip(encoded_extrinsics).for_each(
			|(extrinsic, encoded_extrinsic)| {
				if let Err(e) = Self::apply_checked_extrinsic(extrinsic, encoded_extrinsic) {
					let err: &'static str = e.into();
					panic!("{}", err)
				}
			},
		);

		// post-extrinsics book-keeping
		<frame_system::Pallet<System>>::note_finished_extrinsics();

		frame_executive::Executive::<
			System,
			Block,
			Context,
			UnsignedValidator,
			AllPallets,
			COnRuntimeUpgrade,
		>::idle_and_finalize_hook(block_number);
	}

	/// Actually apply a checked extrinsic given its `encoded_len`; this doesn't note its hash.
	fn apply_checked_extrinsic(
		xt: CheckedOf<Block::Extrinsic, Context>,
		to_note: Vec<u8>,
	) -> ApplyExtrinsicResult {
		sp_io::init_tracing();
		let encoded_len = to_note.len();
		sp_tracing::enter_span!(sp_tracing::info_span!("apply_checked_extrinsic",
					ext=?sp_core::hexdisplay::HexDisplay::from(&to_note)));

		// We don't need to make sure to `note_extrinsic` only after we know it's going to be
		// executed to prevent it from leaking in storage since at this point, it will either
		// execute or panic (and revert storage changes).
		<frame_system::Pallet<System>>::note_extrinsic(to_note);

		// AUDIT: Under no circumstances may this function panic from here onwards.

		// Decode parameters and dispatch
		let dispatch_info = xt.get_dispatch_info();
		let r = Applyable::apply::<UnsignedValidator>(xt, &dispatch_info, encoded_len)?;

		<frame_system::Pallet<System>>::note_applied_extrinsic(&r, dispatch_info);

		Ok(r.map(|_| ()).map_err(|e| e.error))
	}
}
