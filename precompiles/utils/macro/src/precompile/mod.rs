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

#![doc = include_str!("../../docs/precompile_macro.md")]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;
use syn::{parse_macro_input, spanned::Spanned};

pub mod attr;
pub mod expand;
pub mod parse;

pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
	// Macro must be used on `impl` block.
	let mut impl_item = parse_macro_input!(item as syn::ItemImpl);

	// We inspect the block to collect all the data we need for the
	// expansion, and make various checks.
	let precompile = match Precompile::try_from(&mut impl_item) {
		Ok(p) => p,
		Err(e) => return e.into_compile_error().into(),
	};

	// We generate additional code based on the collected data.
	let new_items = precompile.expand();
	let output = quote!(
		#impl_item
		#new_items
	);

	output.into()
}

struct Precompile {
	/// Impl struct type.
	impl_type: syn::Type,

	/// Impl struct ident.
	impl_ident: syn::Ident,

	/// New parsing enum ident.
	enum_ident: syn::Ident,

	/// Generic part that needs to also be used by the input enum.
	generics: syn::Generics,

	/// Which selector corresponds to which variant of the input enum.
	selector_to_variant: BTreeMap<u32, syn::Ident>,

	/// Optional fallback function if no selector matches.
	fallback_to_variant: Option<syn::Ident>,

	/// Describes the content of each variant based on the precompile methods.
	variants_content: BTreeMap<syn::Ident, Variant>,

	/// Since being a precompile set implies lots of changes, we must know it early
	/// in the form of an attribute on the impl block itself.
	tagged_as_precompile_set: bool,

	/// Ident of the function returning the PrecompileSet discriminant.
	precompile_set_discriminant_fn: Option<syn::Ident>,

	/// Type of the PrecompileSet discriminant.
	precompile_set_discriminant_type: Option<syn::Type>,

	/// When generating the selector test the data types might depend on type parameters.
	/// The test thus need to be written using concrete types.
	test_concrete_types: Option<Vec<syn::Type>>,

	/// Ident of a function that performs a check before the call is dispatched to the proper
	/// function.
	pre_check: Option<syn::Ident>,
}

#[derive(Debug, PartialEq, Eq)]
enum Modifier {
	NonPayable,
	Payable,
	View,
}

#[derive(Debug)]
struct Variant {
	/// Description of the arguments of this method, which will also
	/// be members of a struct variant.
	arguments: Vec<Argument>,

	/// String extracted from the selector attribute.
	/// A unit test will be generated to check that this selector matches
	/// the Rust arguments.
	///
	/// > solidity::Codec trait allows to generate this string at runtime only. Thus
	/// > it is required to write it manually in the selector attribute, and
	/// > a unit test is generated to check it matches.
	solidity_arguments_type: String,

	/// Modifier of the function. They are all exclusive and defaults to
	/// `NonPayable`.
	modifier: Modifier,

	/// Selectors of this function to be able to encode back the data.
	/// Empty if it only the fallback function.
	selectors: Vec<u32>,

	/// Output of the variant fn (for better error messages).
	fn_output: syn::Type,
}

#[derive(Debug)]
struct Argument {
	/// Identifier of the argument, which will be used in the struct variant.
	ident: syn::Ident,

	/// Type of the argument, which will be used in the struct variant and
	/// to parse the input.
	ty: syn::Type,
}
