// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

pub mod parachain;
pub mod relay_chain;
pub mod statemint_like;
use cumulus_primitives_core::ParaId;
use polkadot_parachain::primitives::AccountIdConversion;
use sp_runtime::AccountId32;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

use sp_core::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};

pub const PARAALICE: [u8; 20] = [1u8; 20];
pub const RELAYALICE: AccountId32 = AccountId32::new([0u8; 32]);

pub fn para_a_account() -> AccountId32 {
	ParaId::from(1).into_account()
}

pub fn evm_account() -> H160 {
	H160::from_str("1000000000000000000000000000000000000001").unwrap()
}

decl_test_parachain! {
	pub struct ParaA {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(1),
	}
}

decl_test_parachain! {
	pub struct ParaB {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(2),
	}
}

decl_test_parachain! {
	pub struct ParaC {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(3),
	}
}

decl_test_parachain! {
	pub struct Statemint {
		Runtime = statemint_like::Runtime,
		XcmpMessageHandler = statemint_like::MsgQueue,
		DmpMessageHandler = statemint_like::MsgQueue,
		new_ext = statemint_ext(4),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relay_chain::Runtime,
		XcmConfig = relay_chain::XcmConfig,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1, ParaA),
			(2, ParaB),
			(3, ParaC),
			(4, Statemint),
		],
	}
}

pub const INITIAL_BALANCE: u128 = 10_000_000_000_000_000;

pub const INITIAL_EVM_BALANCE: u128 = 0;
pub const INITIAL_EVM_NONCE: u32 = 1;

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use parachain::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(PARAALICE.into(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	// EVM accounts are self-sufficient.
	let mut evm_accounts = BTreeMap::new();
	evm_accounts.insert(
		evm_account(),
		fp_evm::GenesisAccount {
			nonce: U256::from(INITIAL_EVM_NONCE),
			balance: U256::from(INITIAL_EVM_BALANCE),
			storage: Default::default(),
			code: vec![
				0x00, // STOP
			],
		},
	);

	frame_support::traits::GenesisBuild::<Runtime>::assimilate_storage(
		&pallet_evm::GenesisConfig {
			accounts: evm_accounts,
		},
		&mut t,
	)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}

pub fn statemint_ext(para_id: u32) -> sp_io::TestExternalities {
	use statemint_like::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(RELAYALICE.into(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
	use relay_chain::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(RELAYALICE, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;

pub type StatemintBalances = pallet_balances::Pallet<statemint_like::Runtime>;
pub type StatemintChainPalletXcm = pallet_xcm::Pallet<statemint_like::Runtime>;
pub type StatemintAssets = pallet_assets::Pallet<statemint_like::Runtime>;

pub type ParachainPalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type Assets = pallet_assets::Pallet<parachain::Runtime, parachain::ForeignAssetInstance>;
pub type LocalAssets = pallet_assets::Pallet<parachain::Runtime, parachain::LocalAssetInstance>;

pub type Balances = pallet_balances::Pallet<parachain::Runtime>;
pub type Treasury = pallet_treasury::Pallet<parachain::Runtime>;
pub type AssetManager = pallet_asset_manager::Pallet<parachain::Runtime>;
pub type XTokens = orml_xtokens::Pallet<parachain::Runtime>;
pub type RelayBalances = pallet_balances::Pallet<relay_chain::Runtime>;
pub type ParaBalances = pallet_balances::Pallet<parachain::Runtime>;
pub type XcmTransactor = xcm_transactor::Pallet<parachain::Runtime>;
