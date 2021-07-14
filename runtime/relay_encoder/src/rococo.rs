// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use parity_scale_codec::{Decode, Encode};
use sp_runtime::AccountId32;
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
	Proxy(AccountId32, Option<relay_encoder::RelayChainProxyType>),

	#[codec(index = 4u8)]
	// the index should match the position of the dispatchable in the target pallet
	Anonymous(relay_encoder::RelayChainProxyType, u32, u16),
}

pub struct RococoEncoder;

impl relay_encoder::ProxyEncodeCall for RococoEncoder {
	fn encode_call(call: relay_encoder::AvailableProxyCalls) -> Vec<u8> {
		match call {
			relay_encoder::AvailableProxyCalls::CreateAnonymusProxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::Anonymous(a, b, c)).encode()
			}

			relay_encoder::AvailableProxyCalls::Proxy(a, b, c) => {
				let mut call =
					RelayCall::Proxy(AnonymousProxyCall::Proxy(a.clone(), b.clone())).encode();
				// If we encode directly we inject the call length, so we just append the inner call after encoding the outer
				call.append(&mut c.clone());
				call
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rococo::RococoEncoder;
	use frame_support::traits::PalletInfo;
	use relay_encoder::ProxyEncodeCall;

	#[test]
	fn test_proxy_anonymous() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let index = <rococo_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			rococo_runtime::Proxy,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_proxy::Call::<rococo_runtime::Runtime>::anonymous(
			rococo_runtime::ProxyType::Any,
			0,
			0,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<RococoEncoder as ProxyEncodeCall>::encode_call(
				relay_encoder::AvailableProxyCalls::CreateAnonymusProxy(
					relay_encoder::RelayChainProxyType::Any,
					0,
					0
				)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_proxy_proxy() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();
		let index = <rococo_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			rococo_runtime::Proxy,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_proxy::Call::<rococo_runtime::Runtime>::proxy(
			relay_account.clone(),
			None,
			rococo_runtime::Call::Proxy(pallet_proxy::Call::<rococo_runtime::Runtime>::anonymous(
				rococo_runtime::ProxyType::Any,
				0,
				0,
			))
			.into(),
		)
		.encode();
		expected_encoded.append(&mut expected);

		let call_bytes = <RococoEncoder as ProxyEncodeCall>::encode_call(
			relay_encoder::AvailableProxyCalls::CreateAnonymusProxy(
				relay_encoder::RelayChainProxyType::Any,
				0,
				0,
			),
		);

		assert_eq!(
			<RococoEncoder as ProxyEncodeCall>::encode_call(
				relay_encoder::AvailableProxyCalls::Proxy(relay_account, None, call_bytes.into())
			),
			expected_encoded
		);
	}
}
