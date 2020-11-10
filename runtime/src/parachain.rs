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

pub use cumulus_token_dealer;

#[macro_export]
macro_rules! runtime_parachain {
	() => {

		/// This runtime version.
		pub const VERSION: RuntimeVersion = RuntimeVersion {
			spec_name: create_runtime_str!("moonbase-alphanet"),
			impl_name: create_runtime_str!("moonbase-alphanet"),
			authoring_version: 2,
			spec_version: 2,
			impl_version: 3,
			apis: RUNTIME_API_VERSIONS,
			transaction_version: 1,
		};

		impl cumulus_parachain_upgrade::Trait for Runtime {
			type Event = Event;
			type OnValidationFunctionParams = ();
		}

		impl cumulus_message_broker::Trait for Runtime {
			type Event = Event;
			type DownwardMessageHandlers = TokenDealer;
			type UpwardMessage = cumulus_upward_message::RococoUpwardMessage;
			type ParachainId = ParachainInfo;
			type XCMPMessage = cumulus_token_dealer::XCMPMessage<AccountId, Balance>;
			type XCMPMessageHandlers = TokenDealer;
		}

		impl parachain_info::Trait for Runtime {}

		impl cumulus_token_dealer::Trait for Runtime {
			type Event = Event;
			type UpwardMessageSender = MessageBroker;
			type UpwardMessage = cumulus_upward_message::RococoUpwardMessage;
			type Currency = Balances;
			type XCMPMessageSender = MessageBroker;
		}

		// TODO Consensus not supported in parachain
		impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F> {
			fn find_author<'a, I>(_digests: I) -> Option<H160>
			where
				I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
			{
				None
			}
		}

		pub struct PhantomAura;
		impl FindAuthor<u32> for PhantomAura {
			fn find_author<'a, I>(_digests: I) -> Option<u32>
			where
				I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
			{
				Some(0 as u32)
			}
		}

		construct_runtime! {
			pub enum Runtime where
				Block = Block,
				NodeBlock = opaque::Block,
				UncheckedExtrinsic = UncheckedExtrinsic
			{
				System: frame_system::{Module, Call, Storage, Config, Event<T>},
				Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
				Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
				Sudo: pallet_sudo::{Module, Call, Storage, Config<T>, Event<T>},
				RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
				ParachainUpgrade: cumulus_parachain_upgrade::{Module, Call, Storage, Inherent, Event},
				MessageBroker: cumulus_message_broker::{Module, Call, Inherent, Event<T>},
				TransactionPayment: pallet_transaction_payment::{Module, Storage},
				ParachainInfo: parachain_info::{Module, Storage, Config},
				TokenDealer: cumulus_token_dealer::{Module, Call, Event<T>},
				EthereumChainId: pallet_ethereum_chain_id::{Module, Storage, Config},
				EVM: pallet_evm::{Module, Config, Call, Storage, Event<T>},
				Ethereum: pallet_ethereum::{Module, Call, Storage, Event, Config, ValidateUnsigned},
			}
		}
	};
}
