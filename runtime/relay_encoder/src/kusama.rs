// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 24u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
	#[codec(index = 6u8)]
	Stake(StakeCall),
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
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

pub struct KusamaEncoder;

impl relay_encoder::UtilityEncodeCall for KusamaEncoder {
	fn encode_call(call: relay_encoder::AvailableUtilityCalls) -> Vec<u8> {
		match call {
			relay_encoder::AvailableUtilityCalls::AsDerivative(a, b) => {
				let mut call = RelayCall::Utility(UtilityCall::AsDerivative(a.clone())).encode();
				// If we encode directly we inject the call length, so we just append the inner call after encoding the outer
				call.append(&mut b.clone());
				call
			}
		}
	}
}

impl relay_encoder::StakeEncodeCall for KusamaEncoder {
	fn encode_call(call: relay_encoder::AvailableStakeCalls) -> Vec<u8> {
		match call {
			relay_encoder::AvailableStakeCalls::Bond(a, b, c) => {
				RelayCall::Stake(StakeCall::Bond(a.into(), b, c)).encode()
			}

			relay_encoder::AvailableStakeCalls::BondExtra(a) => {
				RelayCall::Stake(StakeCall::BondExtra(a)).encode()
			}

			relay_encoder::AvailableStakeCalls::Unbond(a) => {
				RelayCall::Stake(StakeCall::Unbond(a)).encode()
			}

			relay_encoder::AvailableStakeCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			relay_encoder::AvailableStakeCalls::Validate(a) => {
				RelayCall::Stake(StakeCall::Validate(a)).encode()
			}

			relay_encoder::AvailableStakeCalls::Chill => {
				RelayCall::Stake(StakeCall::Chill).encode()
			}

			relay_encoder::AvailableStakeCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			relay_encoder::AvailableStakeCalls::SetController(a) => {
				RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			relay_encoder::AvailableStakeCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			relay_encoder::AvailableStakeCalls::Nominate(a) => {
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
	use crate::kusama::KusamaEncoder;
	use frame_support::traits::PalletInfo;
	use relay_encoder::{ProxyEncodeCall, StakeEncodeCall};
	use sp_runtime::Perbill;

	#[test]
	fn test_proxy_anonymous() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Proxy,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_proxy::Call::<kusama_runtime::Runtime>::anonymous(
			kusama_runtime::ProxyType::Any,
			0,
			0,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as ProxyEncodeCall>::encode_call(
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
		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Proxy,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_proxy::Call::<kusama_runtime::Runtime>::proxy(
			relay_account.clone(),
			None,
			kusama_runtime::Call::Proxy(pallet_proxy::Call::<kusama_runtime::Runtime>::anonymous(
				kusama_runtime::ProxyType::Any,
				0,
				0,
			))
			.into(),
		)
		.encode();
		expected_encoded.append(&mut expected);

		let call_bytes = <KusamaEncoder as ProxyEncodeCall>::encode_call(
			relay_encoder::AvailableProxyCalls::CreateAnonymusProxy(
				relay_encoder::RelayChainProxyType::Any,
				0,
				0,
			),
		);

		assert_eq!(
			<KusamaEncoder as ProxyEncodeCall>::encode_call(
				relay_encoder::AvailableProxyCalls::Proxy(relay_account, None, call_bytes.into())
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_bond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::bond(
			relay_account.clone().into(),
			100u32.into(),
			pallet_staking::RewardDestination::Controller,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::Bond(
					relay_account.into(),
					100u32.into(),
					pallet_staking::RewardDestination::Controller
				)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_bond_extra() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<kusama_runtime::Runtime>::bond_extra(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::BondExtra(100u32.into(),)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_unbond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<kusama_runtime::Runtime>::unbond(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::Unbond(100u32.into(),)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_withdraw_unbonded() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<kusama_runtime::Runtime>::withdraw_unbonded(100u32).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::WithdrawUnbonded(100u32,)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_validate() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let validator_prefs = pallet_staking::ValidatorPrefs {
			commission: Perbill::from_percent(5),
			blocked: true,
		};

		let mut expected =
			pallet_staking::Call::<kusama_runtime::Runtime>::validate(validator_prefs.clone())
				.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::Validate(validator_prefs)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_nominate() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<kusama_runtime::Runtime>::nominate(vec![relay_account
				.clone()
				.into()])
			.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::Nominate(vec![relay_account.into()])
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_chill() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::chill().encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::Chill
			),
			expected_encoded
		);
	}

	#[test]
	fn test_set_payee() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::set_payee(
			pallet_staking::RewardDestination::Controller,
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::SetPayee(
					pallet_staking::RewardDestination::Controller
				)
			),
			expected_encoded
		);
	}

	#[test]
	fn test_set_controller() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::set_controller(
			relay_account.clone().into(),
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::SetController(relay_account.clone().into())
			),
			expected_encoded
		);
	}
	#[test]
	fn test_rebond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<kusama_runtime::Runtime>::rebond(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				relay_encoder::AvailableStakeCalls::Rebond(100u32.into())
			),
			expected_encoded
		);
	}
}
