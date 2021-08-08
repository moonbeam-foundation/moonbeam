#![cfg_attr(not(feature = "std"), no_std)]
pub mod currencies;

use codec::{Decode, Encode};
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature, RuntimeDebug,
};
use sp_std::{convert::Into, prelude::*};

pub use currencies::{CurrencyId, TokenSymbol};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
