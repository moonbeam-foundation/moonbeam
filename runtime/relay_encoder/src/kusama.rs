// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_runtime::Perbill;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 91u8)]
	// the index should match the position of the module in `construct_runtime!`
	Proxy(AnonymousProxyCall),
}

#[derive(Encode, Decode)]
pub enum AnonymousProxyCall {
	#[codec(index = 0u8)]
	Proxy(
		AccountId32,
		Option<relay_encoder::RelayChainProxyType>,
		Vec<u8>,
	),

	#[codec(index = 4u8)]
	// the index should match the position of the dispatchable in the target pallet
	Anonymous(relay_encoder::RelayChainProxyType, u32, u16),
}

pub struct PolkadotEncoder;

impl relay_encoder::EncodeCall for PolkadotEncoder {
	fn encode_call(call: relay_encoder::AvailableCalls) -> Vec<u8> {
		match call {
			relay_encoder::AvailableCalls::CreateAnonymusProxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::Anonymous(a, b, c)).encode()
			}

			relay_encoder::AvailableCalls::Proxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::Proxy(a, b, c)).encode()
			}
			_ => panic!("Unimplemented"),
		}
	}
}
