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

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;
use syn::{parse_macro_input, spanned::Spanned};

pub mod attr;
pub mod expand;
pub mod parse;

pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as syn::AttributeArgs);
	let mut impl_item = parse_macro_input!(item as syn::ItemImpl);

	let precompile = match Precompile::try_from(args, &mut impl_item) {
		Ok(p) => p,
		Err(e) => return e.into_compile_error().into(),
	};

	let enum_ = precompile.generate_enum();
	let enum_impl = precompile.generate_enum_impl();
	let precomp_impl = precompile.generate_precompile_impl();
	let test_signature = precompile.generate_test_solidity_signature();

	let output = quote! {
		#impl_item
		#enum_
		#enum_impl
		#precomp_impl
		#test_signature
	};

	output.into()
}

pub struct Precompile {
	/// Impl struct type.
	struct_type: syn::Type,

	/// Impl struct type.
	struct_ident: syn::Ident,

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
}

#[derive(Debug, PartialEq, Eq)]
pub enum Modifier {
	NonPayable,
	Payable,
	View,
}

#[derive(Debug)]
pub struct Variant {
	/// Description of the arguments of this method, which will also
	/// be members of a struct variant.
	arguments: Vec<Argument>,

	/// String extracted from the selector attribute.
	/// A unit test will be generated to check that this selector matches
	/// the Rust arguments.
	///
	/// > EvmData trait allows to generate this string at runtime only. Thus
	/// > it is required to write it manually in the selector attribute, and
	/// > a unit test is generated to check it matches.
	solidity_arguments_type: String,

	/// Modifier of the function. They are all exclusive and defaults to
	/// `NonPayable`.
	modifier: Modifier,

	/// One of the selectors to be able to encode back the data.
	/// None if it only the fallback function.
	encode_selector: Option<u32>,
}

#[derive(Debug)]
pub struct Argument {
	/// Identifier of the argument, which will be used in the struct variant.
	ident: syn::Ident,

	/// Type of the argument, which will be used in the struct variant and
	/// to parse the input.
	ty: syn::Type,
}
