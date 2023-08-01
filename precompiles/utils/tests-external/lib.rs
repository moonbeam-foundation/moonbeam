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

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use evm::Context;
	use fp_evm::{ExitReason, ExitRevert, PrecompileFailure, PrecompileHandle};
	use frame_support::traits::Everything;
	use frame_support::{construct_runtime, parameter_types, weights::Weight};
	use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
	use precompile_utils::{
		precompile_set::*,
		solidity::{codec::Writer, revert::revert},
		testing::*,
		EvmResult,
	};
	use sp_core::H160;
	use sp_core::{H256, U256};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};

	pub type AccountId = MockAccount;
	pub type Balance = u128;
	pub type BlockNumber = u32;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
	type Block = frame_system::mocking::MockBlock<Runtime>;

	construct_runtime!(
		pub enum Runtime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
			Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
			Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u32 = 250;
		pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
		pub const SS58Prefix: u8 = 42;
	}

	impl frame_system::Config for Runtime {
		type BaseCallFilter = Everything;
		type DbWeight = ();
		type RuntimeOrigin = RuntimeOrigin;
		type Index = u64;
		type BlockNumber = BlockNumber;
		type RuntimeCall = RuntimeCall;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
		type RuntimeEvent = RuntimeEvent;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}
	parameter_types! {
		pub const ExistentialDeposit: u128 = 0;
	}
	impl pallet_balances::Config for Runtime {
		type MaxReserves = ();
		type ReserveIdentifier = [u8; 4];
		type MaxLocks = ();
		type Balance = Balance;
		type RuntimeEvent = RuntimeEvent;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type HoldIdentifier = ();
		type FreezeIdentifier = ();
		type MaxHolds = ();
		type MaxFreezes = ();
	}

	#[derive(Debug, Clone)]
	pub struct MockPrecompile;

	#[precompile_utils::precompile]
	impl MockPrecompile {
		// a3cab0dd
		#[precompile::public("subcall()")]
		fn subcall(handle: &mut impl PrecompileHandle) -> EvmResult {
			match handle.call(
				handle.code_address(),
				None,
				// calls subcallLayer2()
				Writer::new_with_selector(0x0b93381bu32).build(),
				None,
				false,
				&Context {
					caller: handle.code_address(),
					address: handle.code_address(),
					apparent_value: 0.into(),
				},
			) {
				(ExitReason::Succeed(_), _) => Ok(()),
				(ExitReason::Revert(_), v) => Err(PrecompileFailure::Revert {
					exit_status: ExitRevert::Reverted,
					output: v,
				}),
				_ => Err(revert("unexpected error")),
			}
		}

		// 0b93381b
		#[precompile::public("success()")]
		fn success(_: &mut impl PrecompileHandle) -> EvmResult {
			Ok(())
		}
	}

	struct MockPrecompileHandle;
	impl PrecompileHandle for MockPrecompileHandle {
		fn call(
			&mut self,
			_: sp_core::H160,
			_: Option<evm::Transfer>,
			_: Vec<u8>,
			_: Option<u64>,
			_: bool,
			_: &evm::Context,
		) -> (evm::ExitReason, Vec<u8>) {
			unimplemented!()
		}

		fn record_cost(&mut self, _: u64) -> Result<(), evm::ExitError> {
			Ok(())
		}

		fn remaining_gas(&self) -> u64 {
			unimplemented!()
		}

		fn log(
			&mut self,
			_: sp_core::H160,
			_: Vec<sp_core::H256>,
			_: Vec<u8>,
		) -> Result<(), evm::ExitError> {
			unimplemented!()
		}

		fn code_address(&self) -> sp_core::H160 {
			unimplemented!()
		}

		fn input(&self) -> &[u8] {
			unimplemented!()
		}

		fn context(&self) -> &evm::Context {
			unimplemented!()
		}

		fn is_static(&self) -> bool {
			true
		}

		fn gas_limit(&self) -> Option<u64> {
			unimplemented!()
		}

		fn record_external_cost(
			&mut self,
			_ref_time: Option<u64>,
			_proof_size: Option<u64>,
		) -> Result<(), fp_evm::ExitError> {
			Ok(())
		}

		fn refund_external_cost(&mut self, _ref_time: Option<u64>, _proof_size: Option<u64>) {}
	}

	pub type Precompiles<R> = PrecompileSetBuilder<
		R,
		(
			PrecompileAt<AddressU64<1>, MockPrecompile>,
			PrecompileAt<AddressU64<2>, MockPrecompile, CallableByContract>,
			PrecompileAt<AddressU64<3>, MockPrecompile, CallableByPrecompile>,
			PrecompileAt<AddressU64<4>, MockPrecompile, SubcallWithMaxNesting<1>>,
		),
	>;

	pub type PCall = MockPrecompileCall;

	const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;

	parameter_types! {
		pub BlockGasLimit: U256 = U256::from(u64::MAX);
		pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
		pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
		pub GasLimitPovSizeRatio: u64 = {
			let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
			block_gas_limit.saturating_div(MAX_POV_SIZE)
		};
	}

	impl pallet_evm::Config for Runtime {
		type FeeCalculator = ();
		type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
		type WeightPerGas = WeightPerGas;
		type CallOrigin = EnsureAddressRoot<AccountId>;
		type WithdrawOrigin = EnsureAddressNever<AccountId>;
		type AddressMapping = AccountId;
		type Currency = Balances;
		type RuntimeEvent = RuntimeEvent;
		type Runner = pallet_evm::runner::stack::Runner<Self>;
		type PrecompilesType = Precompiles<Runtime>;
		type PrecompilesValue = PrecompilesValue;
		type ChainId = ();
		type OnChargeTransaction = ();
		type BlockGasLimit = BlockGasLimit;
		type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
		type FindAuthor = ();
		type OnCreate = ();
		type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
		type Timestamp = Timestamp;
		type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
	}

	parameter_types! {
		pub const MinimumPeriod: u64 = 5;
	}
	impl pallet_timestamp::Config for Runtime {
		type Moment = u64;
		type OnTimestampSet = ();
		type MinimumPeriod = MinimumPeriod;
		type WeightInfo = ();
	}

	struct ExtBuilder;

	impl Default for ExtBuilder {
		fn default() -> ExtBuilder {
			ExtBuilder
		}
	}

	impl ExtBuilder {
		#[cfg(test)]
		fn build(self) -> sp_io::TestExternalities {
			let t = frame_system::GenesisConfig::default()
				.build_storage::<Runtime>()
				.expect("Frame system builds valid default genesis config");

			let mut ext = sp_io::TestExternalities::new(t);
			ext.execute_with(|| {
				System::set_block_number(1);
			});
			ext
		}
	}

	#[cfg(test)]
	fn precompiles() -> Precompiles<Runtime> {
		PrecompilesValue::get()
	}

	#[test]
	fn default_checks_succeed_when_called_by_eoa() {
		ExtBuilder::default().build().execute_with(|| {
			precompiles()
				.prepare_test(Alice, H160::from_low_u64_be(1), PCall::success {})
				.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
				.execute_returns(())
		})
	}

	#[test]
	fn default_checks_revert_when_called_by_precompile() {
		ExtBuilder::default().build().execute_with(|| {
			precompiles()
				.prepare_test(
					H160::from_low_u64_be(1),
					H160::from_low_u64_be(1),
					PCall::success {},
				)
				.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
				.execute_reverts(|r| r == b"Function not callable by precompiles")
		})
	}

	#[test]
	fn default_checks_revert_when_called_by_contract() {
		ExtBuilder::default().build().execute_with(|| {
			pallet_evm::Pallet::<Runtime>::create_account(
				Alice.into(),
				hex_literal::hex!("1460006000fd").to_vec(),
			);

			precompiles()
				.prepare_test(Alice, H160::from_low_u64_be(1), PCall::success {})
				.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
				.execute_reverts(|r| r == b"Function not callable by smart contracts")
		})
	}

	#[test]
	fn default_checks_revert_when_doing_subcall() {
		ExtBuilder::default().build().execute_with(|| {
			precompiles()
				.prepare_test(Alice, H160::from_low_u64_be(1), PCall::subcall {})
				.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
				.execute_reverts(|r| r == b"subcalls disabled for this precompile")
		})
	}

	#[test]
	fn callable_by_contract_works() {
		ExtBuilder::default().build().execute_with(|| {
			pallet_evm::Pallet::<Runtime>::create_account(
				Alice.into(),
				hex_literal::hex!("1460006000fd").to_vec(),
			);

			precompiles()
				.prepare_test(Alice, H160::from_low_u64_be(2), PCall::success {})
				.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
				.execute_returns(())
		})
	}

	#[test]
	fn callable_by_precompile_works() {
		ExtBuilder::default().build().execute_with(|| {
			precompiles()
				.prepare_test(
					H160::from_low_u64_be(3),
					H160::from_low_u64_be(3),
					PCall::success {},
				)
				.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
				.execute_returns(())
		})
	}

	#[test]
	fn subcalls_works_when_allowed() {
		ExtBuilder::default().build().execute_with(|| {
			let subcall_occured = Rc::new(RefCell::new(false));
			{
				let subcall_occured = Rc::clone(&subcall_occured);
				precompiles()
					.prepare_test(Alice, H160::from_low_u64_be(4), PCall::subcall {})
					.with_subcall_handle(move |Subcall { .. }| {
						*subcall_occured.borrow_mut() = true;
						SubcallOutput::succeed()
					})
					.execute_returns(());
			}
			assert!(*subcall_occured.borrow());
		})
	}

	#[test]
	fn get_address_type_works_for_eoa() {
		ExtBuilder::default().build().execute_with(|| {
			let addr = H160::repeat_byte(0x1d);
			assert_eq!(
				AddressType::EOA,
				get_address_type::<Runtime>(&mut MockPrecompileHandle, addr).expect("OOG")
			);
		})
	}

	#[test]
	fn get_address_type_works_for_precompile() {
		ExtBuilder::default().build().execute_with(|| {
			let addr = H160::repeat_byte(0x1d);
			pallet_evm::AccountCodes::<Runtime>::insert(addr, vec![0x60, 0x00, 0x60, 0x00, 0xfd]);
			assert_eq!(
				AddressType::Precompile,
				get_address_type::<Runtime>(&mut MockPrecompileHandle, addr).expect("OOG")
			);
		})
	}

	#[test]
	fn get_address_type_works_for_smart_contract() {
		ExtBuilder::default().build().execute_with(|| {
			let addr = H160::repeat_byte(0x1d);

			// length > 5
			pallet_evm::AccountCodes::<Runtime>::insert(
				addr,
				vec![0x60, 0x00, 0x60, 0x00, 0xfd, 0xff, 0xff],
			);
			assert_eq!(
				AddressType::Contract,
				get_address_type::<Runtime>(&mut MockPrecompileHandle, addr).expect("OOG")
			);

			// length < 5
			pallet_evm::AccountCodes::<Runtime>::insert(addr, vec![0x60, 0x00, 0x60]);
			assert_eq!(
				AddressType::Contract,
				get_address_type::<Runtime>(&mut MockPrecompileHandle, addr).expect("OOG")
			);
		})
	}

	#[test]
	fn get_address_type_works_for_unknown() {
		ExtBuilder::default().build().execute_with(|| {
			let addr = H160::repeat_byte(0x1d);
			pallet_evm::AccountCodes::<Runtime>::insert(addr, vec![0x11, 0x00, 0x60, 0x00, 0xfd]);
			assert_eq!(
				AddressType::Unknown,
				get_address_type::<Runtime>(&mut MockPrecompileHandle, addr).expect("OOG")
			);
		})
	}
}
