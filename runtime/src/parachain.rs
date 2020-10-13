pub use cumulus_token_dealer;

#[macro_export]
macro_rules! runtime_parachain {
    () => {
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
                EVM: pallet_evm::{Module, Config, Call, Storage, Event<T>},
                Ethereum: pallet_ethereum::{Module, Call, Storage, Event, Config, ValidateUnsigned},
            }
        }
    };
}