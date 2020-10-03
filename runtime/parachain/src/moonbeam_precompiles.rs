
use sp_std::{prelude::*};
use sp_core::H160;

pub struct MoonbeamPrecompiles;

/// Linear gas cost
fn ensure_linear_cost(
	target_gas: Option<usize>,
	len: usize,
	base: usize,
	word: usize
) -> Result<usize, pallet_evm::ExitError> {
	let cost = base.checked_add(
		word.checked_mul(len.saturating_add(31) / 32).ok_or(pallet_evm::ExitError::OutOfGas)?
	).ok_or(pallet_evm::ExitError::OutOfGas)?;
	if let Some(target_gas) = target_gas {
		if cost > target_gas {
			return Err(pallet_evm::ExitError::OutOfGas)
		}
	}
	Ok(cost)
}

// prepends "deadbeef" to any data provided
struct DeadbeefPrecompiled;

impl pallet_evm::Precompile for DeadbeefPrecompiled {
	fn execute(
		input: &[u8],
		target_gas: Option<usize>
	) -> core::result::Result<(pallet_evm::ExitSucceed, Vec<u8>, usize), pallet_evm::ExitError> {
		let cost = ensure_linear_cost(target_gas, input.len(), 15, 3)?;

		log::info!("Calling deadbeef precompiled contract");

		let mut result_vec = hex_literal::hex!("deadbeef").to_vec();
		result_vec.extend(input.to_vec());

		Ok((pallet_evm::ExitSucceed::Returned, result_vec, cost))
	}
}

type PrecompiledCallable = fn(&[u8], Option<usize>)
	-> core::result::Result<(pallet_evm::ExitSucceed, Vec<u8>, usize), pallet_evm::ExitError>;

fn get_precompiled_func_from_address(address: &H160) -> Option<PrecompiledCallable> {
	use core::str::FromStr;
	use pallet_evm::Precompile;

	// Note that addresses from_str should not start with 0x, just the hex value
	let addr_deadbeef = H160::from_str("0000000000000000000000000000000000001000").expect("Invalid address at precompiles generation");

	if *address == addr_deadbeef {
		return Some(DeadbeefPrecompiled::execute);
	}

	None
}

impl pallet_evm::Precompiles for MoonbeamPrecompiles {
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<usize>
	) -> Option<core::result::Result<(pallet_evm::ExitSucceed, Vec<u8>, usize), pallet_evm::ExitError>> {
		match get_precompiled_func_from_address(&address) {
		   Some(func) => return Some(func(input, target_gas)),
		   _ => {},
		};

		None
	}
}

