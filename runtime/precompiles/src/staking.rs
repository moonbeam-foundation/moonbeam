use evm::{Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use pallet_evm::AddressMapping;
use pallet_evm::GasWeightMapping;
use pallet_evm::Precompile;
use sp_core::H160;
use sp_core::U256;
use sp_std::convert::TryFrom;
use sp_std::convert::TryInto;
use sp_std::fmt::Debug;
use sp_std::{marker::PhantomData, vec::Vec};

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from parachain_staking.
///
/// Currently it only supports nominate. More to come later.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct ParachainStakingWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		// Basic sanity checking for length
		// https://solidity-by-example.org/primitives/
		const COLLATOR_SIZE_BYTES: usize = 20;
		const AMOUNT_SIZE_BYTES: usize = 32;
		const TOTAL_SIZE_BYTES: usize = COLLATOR_SIZE_BYTES + AMOUNT_SIZE_BYTES;

		if input.len() != TOTAL_SIZE_BYTES {
			return Err(ExitError::Other(
				"input length for Sacrifice must be exactly 16 bytes".into(),
			));
		}

		// Convert to right data types
		//TODO This precompile will not work in runtimes that use a differet AccountId type
		// The trouble is how to specify the nomination target from the evm when you don't know
		// its size. I guess the general solution is to accept a size parameter, decode to AccountId
		// But that would make the solidity experience a lot less nice. Maybe its best to keep the
		// AccountId: From<H160> requirement.
		let collator = H160::from_slice(&input[0..COLLATOR_SIZE_BYTES]);

		let amount: BalanceOf<Runtime> =
			sp_core::U256::from_big_endian(&input[COLLATOR_SIZE_BYTES..TOTAL_SIZE_BYTES])
				.try_into()
				.map_err(|_| {
					ExitError::Other("amount is too large for Runtime's balance type".into())
				})?;

		println!("Collator account is {:?}", collator);
		println!("Amount is {:?}", amount);

		// Construct a call
		let inner_call = parachain_staking::Call::<Runtime>::nominate(collator.into(), amount);
		let outer_call: Runtime::Call = inner_call.into();
		let info = outer_call.get_dispatch_info();

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			let valid_weight = info.weight <= Runtime::GasWeightMapping::gas_to_weight(gas_limit);
			if !valid_weight {
				return Err(ExitError::OutOfGas);
			}
		}

		// Dispatch that call
		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		match outer_call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let gas_used = Runtime::GasWeightMapping::weight_to_gas(
					post_info.actual_weight.unwrap_or(info.weight),
				);
				Ok((ExitSucceed::Stopped, Default::default(), gas_used))
			}
			Err(_) => Err(ExitError::Other(
				"Parachain staking nomination failed".into(),
			)),
		}
	}
}
