// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 29u8)]
	// the index should match the position of the module in `construct_runtime!`
	Proxy(AnonymousProxyCall),
	#[codec(index = 7u8)]
	Stake(StakeCall),
}

#[derive(Encode, Decode)]
pub enum AnonymousProxyCall {
	#[codec(index = 0u8)]
	Proxy(AccountId32, Option<relay_encoder::RelayChainProxyType>),

	#[codec(index = 4u8)]
	// the index should match the position of the dispatchable in the target pallet
	Anonymous(relay_encoder::RelayChainProxyType, u32, u16),
}

#[derive(Encode, Decode)]
pub enum StakeCall {
	#[codec(index = 0u16)]
	// the index should match the position of the dispatchable in the target pallet
	Bond(
		<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source,
		#[codec(compact)] cumulus_primitives_core::relay_chain::Balance,
		pallet_staking::RewardDestination<AccountId32>,
	),
	#[codec(index = 1u16)]
	BondExtra(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
	#[codec(index = 2u16)]
	Unbond(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
	#[codec(index = 3u16)]
	WithdrawUnbonded(u32),
	#[codec(index = 4u16)]
	Validate(pallet_staking::ValidatorPrefs),
	#[codec(index = 5u16)]
	Nominate(Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source>),
	#[codec(index = 6u16)]
	Chill,
	#[codec(index = 7u16)]
	SetPayee(pallet_staking::RewardDestination<AccountId32>),
	#[codec(index = 8u16)]
	SetController(<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source),
	#[codec(index = 19u16)]
	Rebond(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
}

pub struct PolkadotEncoder;

impl relay_encoder::EncodeCall for PolkadotEncoder {
	fn encode_call(call: relay_encoder::AvailableCalls) -> Vec<u8> {
		match call {
			relay_encoder::AvailableCalls::CreateAnonymusProxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::Anonymous(a, b, c)).encode()
			}

			relay_encoder::AvailableCalls::Proxy(a, b, c) => {
				let mut call =
					RelayCall::Proxy(AnonymousProxyCall::Proxy(a.clone(), b.clone())).encode();
				// If we encode directly we inject the call length, so we just append the inner call after encoding the outer
				call.append(&mut c.clone());
				call
			}

			relay_encoder::AvailableCalls::Bond(a, b, c) => {
				RelayCall::Stake(StakeCall::Bond(a.into(), b, c)).encode()
			}

			relay_encoder::AvailableCalls::BondExtra(a) => {
				RelayCall::Stake(StakeCall::BondExtra(a)).encode()
			}

			relay_encoder::AvailableCalls::Unbond(a) => {
				RelayCall::Stake(StakeCall::Unbond(a)).encode()
			}

			relay_encoder::AvailableCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			relay_encoder::AvailableCalls::Validate(a) => {
				RelayCall::Stake(StakeCall::Validate(a)).encode()
			}

			relay_encoder::AvailableCalls::Chill => RelayCall::Stake(StakeCall::Chill).encode(),

			relay_encoder::AvailableCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			relay_encoder::AvailableCalls::SetController(a) => {
				RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			relay_encoder::AvailableCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			relay_encoder::AvailableCalls::Nominate(a) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					a.iter().map(|add| (*add).clone().into()).collect();

				RelayCall::Stake(StakeCall::Nominate(nominated)).encode()
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::polkadot::PolkadotEncoder;
	use frame_support::traits::PalletInfo;
	use relay_encoder::EncodeCall;
	use sp_runtime::Perbill;

	#[test]
	fn test_proxy_anonymous() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Proxy,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_proxy::Call::<polkadot_runtime::Runtime>::anonymous(
			polkadot_runtime::ProxyType::Any,
			0,
			0,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::CreateAnonymusProxy(
				relay_encoder::RelayChainProxyType::Any,
				0,
				0
			)),
			expected_encoded
		);
	}
	#[test]
	fn test_proxy_proxy() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();
		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Proxy,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_proxy::Call::<polkadot_runtime::Runtime>::proxy(
			relay_account.clone(),
			None,
			polkadot_runtime::Call::Proxy(
				pallet_proxy::Call::<polkadot_runtime::Runtime>::anonymous(
					polkadot_runtime::ProxyType::Any,
					0,
					0,
				),
			)
			.into(),
		)
		.encode();
		expected_encoded.append(&mut expected);

		let call_bytes =
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::CreateAnonymusProxy(
				relay_encoder::RelayChainProxyType::Any,
				0,
				0,
			));

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Proxy(
				relay_account,
				None,
				call_bytes.into()
			)),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_bond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<polkadot_runtime::Runtime>::bond(
			relay_account.clone().into(),
			100u32.into(),
			pallet_staking::RewardDestination::Controller,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Bond(
				relay_account.into(),
				100u32.into(),
				pallet_staking::RewardDestination::Controller
			)),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_bond_extra() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<polkadot_runtime::Runtime>::bond_extra(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::BondExtra(100u32.into(),)),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_unbond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<polkadot_runtime::Runtime>::unbond(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Unbond(100u32.into(),)),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_withdraw_unbonded() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<polkadot_runtime::Runtime>::withdraw_unbonded(100u32).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::WithdrawUnbonded(100u32,)),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_validate() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let validator_prefs = pallet_staking::ValidatorPrefs {
			commission: Perbill::from_percent(5),
			blocked: true,
		};

		let mut expected =
			pallet_staking::Call::<polkadot_runtime::Runtime>::validate(validator_prefs.clone())
				.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Validate(validator_prefs)),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_nominate() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<polkadot_runtime::Runtime>::nominate(vec![relay_account
				.clone()
				.into()])
			.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Nominate(vec![
				relay_account.into()
			])),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_chill() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<polkadot_runtime::Runtime>::chill().encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Chill),
			expected_encoded
		);
	}

	#[test]
	fn test_set_payee() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<polkadot_runtime::Runtime>::set_payee(
			pallet_staking::RewardDestination::Controller,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::SetPayee(
				pallet_staking::RewardDestination::Controller
			)),
			expected_encoded
		);
	}

	#[test]
	fn test_set_controller() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<polkadot_runtime::Runtime>::set_controller(
			relay_account.clone().into(),
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::SetController(
				relay_account.clone().into()
			)),
			expected_encoded
		);
	}
	#[test]
	fn test_rebond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <polkadot_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			polkadot_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<polkadot_runtime::Runtime>::rebond(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			PolkadotEncoder::encode_call(relay_encoder::AvailableCalls::Rebond(100u32.into())),
			expected_encoded
		);
	}
}
