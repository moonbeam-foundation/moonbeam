// Copyright 2019-2022 PureStake Inc.
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

#![crate_type = "proc-macro"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use sha3::{Digest, Keccak256};
use syn::{parse_macro_input, spanned::Spanned, Expr, Ident, ItemType, Lit, LitStr};

mod derive_codec;
mod precompile;
mod precompile_name_from_address;

struct Bytes(Vec<u8>);

impl ::std::fmt::Debug for Bytes {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> ::std::fmt::Result {
		let data = &self.0;
		write!(f, "[")?;
		if !data.is_empty() {
			write!(f, "{:#04x}u8", data[0])?;
			for unit in data.iter().skip(1) {
				write!(f, ", {:#04x}", unit)?;
			}
		}
		write!(f, "]")
	}
}

#[proc_macro]
pub fn keccak256(input: TokenStream) -> TokenStream {
	let lit_str = parse_macro_input!(input as LitStr);

	let hash = Keccak256::digest(lit_str.value().as_bytes());

	let bytes = Bytes(hash.to_vec());
	let eval_str = format!("{:?}", bytes);
	let eval_ts: proc_macro2::TokenStream = eval_str.parse().unwrap_or_else(|_| {
		panic!(
			"Failed to parse the string \"{}\" to TokenStream.",
			eval_str
		);
	});
	quote!(#eval_ts).into()
}

#[proc_macro_attribute]
pub fn precompile(attr: TokenStream, input: TokenStream) -> TokenStream {
	precompile::main(attr, input)
}

#[proc_macro_attribute]
pub fn precompile_name_from_address(attr: TokenStream, input: TokenStream) -> TokenStream {
	precompile_name_from_address::main(attr, input)
}

#[proc_macro_derive(Codec)]
pub fn derive_codec(input: TokenStream) -> TokenStream {
	derive_codec::main(input)
}
