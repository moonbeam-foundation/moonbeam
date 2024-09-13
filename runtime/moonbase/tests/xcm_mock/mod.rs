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
use pallet_xcm_transactor::relay_indices::*;
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::{AccountId32, BuildStorage};
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};

use polkadot_runtime_parachains::configuration::{
	GenesisConfig as ConfigurationGenesisConfig, HostConfiguration,
};
use polkadot_runtime_parachains::paras::{
	GenesisConfig as ParasGenesisConfig, ParaGenesisArgs, ParaKind,
};

use sp_core::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};

pub const PARAALICE: [u8; 20] = [1u8; 20];
pub const RELAYALICE: AccountId32 = AccountId32::new([0u8; 32]);
pub const RELAYBOB: AccountId32 = AccountId32::new([2u8; 32]);

pub fn para_a_account() -> AccountId32 {
	ParaId::from(1).into_account_truncating()
}

pub fn para_b_account() -> AccountId32 {
	ParaId::from(2).into_account_truncating()
}

pub fn para_a_account_20() -> parachain::AccountId {
	ParaId::from(1).into_account_truncating()
}

pub fn evm_account() -> H160 {
	H160::from_str("1000000000000000000000000000000000000001").unwrap()
}

pub fn mock_para_genesis_info() -> ParaGenesisArgs {
	ParaGenesisArgs {
		genesis_head: vec![1u8].into(),
		validation_code: vec![1u8].into(),
		para_kind: ParaKind::Parachain,
	}
}

pub fn mock_relay_config() -> HostConfiguration<relay_chain::BlockNumber> {
	HostConfiguration::<relay_chain::BlockNumber> {
		hrmp_channel_max_capacity: u32::MAX,
		hrmp_channel_max_total_size: u32::MAX,
		hrmp_max_parachain_inbound_channels: 10,
		hrmp_max_parachain_outbound_channels: 10,
		hrmp_channel_max_message_size: u32::MAX,
		// Changed to avoid aritmetic errors within hrmp_close
		max_downward_message_size: 100_000u32,
		..Default::default()
	}
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
		new_ext = statemint_ext(1000),
	}
}

decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relay_chain::Runtime,
		RuntimeCall = relay_chain::RuntimeCall,
		RuntimeEvent = relay_chain::RuntimeEvent,
		XcmConfig = relay_chain::XcmConfig,
		MessageQueue = relay_chain::MessageQueue,
		System = relay_chain::System,
		new_ext = relay_ext(vec![1, 2, 3, 1000]),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(1, ParaA),
			(2, ParaB),
			(3, ParaC),
			(1000, Statemint),
		],
	}
}

pub const INITIAL_BALANCE: u128 = 10_000_000_000_000_000;

pub const INITIAL_EVM_BALANCE: u128 = 0;
pub const INITIAL_EVM_NONCE: u32 = 1;

pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use parachain::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(PARAALICE.into(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	pallet_xcm_transactor::GenesisConfig::<Runtime> {
		// match relay runtime construct_runtime order in xcm_mock::relay_chain
		relay_indices: RelayChainIndices {
			hrmp: 6u8,
			init_open_channel: 0u8,
			accept_open_channel: 1u8,
			close_channel: 2u8,
			cancel_open_request: 6u8,
			..Default::default()
		},
		..Default::default()
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

	let genesis_config = pallet_evm::GenesisConfig::<Runtime> {
		accounts: evm_accounts,
		..Default::default()
	};
	genesis_config.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}

pub fn statemint_ext(para_id: u32) -> sp_io::TestExternalities {
	use statemint_like::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(RELAYALICE.into(), INITIAL_BALANCE),
			(RELAYBOB.into(), INITIAL_BALANCE),
		],
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

pub fn relay_ext(paras: Vec<u32>) -> sp_io::TestExternalities {
	use relay_chain::{Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(RELAYALICE, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let para_genesis: Vec<(ParaId, ParaGenesisArgs)> = paras
		.iter()
		.map(|&para_id| (para_id.into(), mock_para_genesis_info()))
		.collect();

	let genesis_config = ConfigurationGenesisConfig::<Runtime> {
		config: mock_relay_config(),
	};
	genesis_config.assimilate_storage(&mut t).unwrap();

	let genesis_config = ParasGenesisConfig::<Runtime> {
		paras: para_genesis,
		..Default::default()
	};
	genesis_config.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}
pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type Hrmp = polkadot_runtime_parachains::hrmp::Pallet<relay_chain::Runtime>;

pub type StatemintBalances = pallet_balances::Pallet<statemint_like::Runtime>;
pub type StatemintChainPalletXcm = pallet_xcm::Pallet<statemint_like::Runtime>;
pub type StatemintAssets = pallet_assets::Pallet<statemint_like::Runtime>;

pub type ParachainPalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type Assets = pallet_assets::Pallet<parachain::Runtime, parachain::ForeignAssetInstance>;

pub type Treasury = pallet_treasury::Pallet<parachain::Runtime>;
pub type AssetManager = pallet_asset_manager::Pallet<parachain::Runtime>;
pub type XTokens = orml_xtokens::Pallet<parachain::Runtime>;
pub type RelayBalances = pallet_balances::Pallet<relay_chain::Runtime>;
pub type ParaBalances = pallet_balances::Pallet<parachain::Runtime>;
pub type XcmTransactor = pallet_xcm_transactor::Pallet<parachain::Runtime>;
pub type XcmWeightTrader = pallet_xcm_weight_trader::Pallet<parachain::Runtime>;
