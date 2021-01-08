#![cfg_attr(not(feature = "std"), no_std)]

use ethereum::Transaction;
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait TxPoolRuntimeApi {
		fn extrinsic_filter(xt: Vec<<Block as BlockT>::Extrinsic>) -> Vec<Transaction>;
	}
}
