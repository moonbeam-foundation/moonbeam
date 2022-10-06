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

use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;

pub fn take_attributes<A>(attributes: &mut Vec<syn::Attribute>) -> syn::Result<Vec<A>>
where
	A: syn::parse::Parse,
{
	let mut output = vec![];
	let pred = |attr: &syn::Attribute| {
		attr.path
			.segments
			.first()
			.map_or(false, |segment| segment.ident == "precompile")
	};

	while let Some(index) = attributes.iter().position(pred) {
		let attr = attributes.remove(index);
		let attr = syn::parse2(attr.into_token_stream())?;
		output.push(attr)
	}
	Ok(output)
}

/// List of additional token to be used for parsing.
pub mod keyword {
	syn::custom_keyword!(precompile);
	syn::custom_keyword!(public);
	syn::custom_keyword!(fallback);
	syn::custom_keyword!(payable);
	syn::custom_keyword!(view);
	syn::custom_keyword!(discriminant);
	syn::custom_keyword!(precompile_set);
	syn::custom_keyword!(test_concrete_types);
	syn::custom_keyword!(pre_check);
}

/// Attributes for methods.
pub enum MethodAttr {
	Public(Span, syn::LitStr),
	Fallback(Span),
	Payable(Span),
	View(Span),
	Discriminant(Span),
	PreCheck(Span),
}

impl syn::parse::Parse for MethodAttr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		input.parse::<syn::Token![#]>()?;
		let content;
		syn::bracketed!(content in input);
		content.parse::<keyword::precompile>()?;
		content.parse::<syn::Token![::]>()?;

		let lookahead = content.lookahead1();

		if lookahead.peek(keyword::public) {
			let span = content.parse::<keyword::public>()?.span();

			let inner;
			syn::parenthesized!(inner in content);
			let signature = inner.parse::<syn::LitStr>()?;

			Ok(MethodAttr::Public(span, signature))
		} else if lookahead.peek(keyword::fallback) {
			Ok(MethodAttr::Fallback(
				content.parse::<keyword::fallback>()?.span(),
			))
		} else if lookahead.peek(keyword::payable) {
			Ok(MethodAttr::Payable(
				content.parse::<keyword::payable>()?.span(),
			))
		} else if lookahead.peek(keyword::view) {
			Ok(MethodAttr::View(content.parse::<keyword::view>()?.span()))
		} else if lookahead.peek(keyword::discriminant) {
			Ok(MethodAttr::Discriminant(
				content.parse::<keyword::discriminant>()?.span(),
			))
		} else if lookahead.peek(keyword::pre_check) {
			Ok(MethodAttr::PreCheck(
				content.parse::<keyword::pre_check>()?.span(),
			))
		} else {
			Err(lookahead.error())
		}
	}
}

/// Attributes for the main impl Block.
pub enum ImplAttr {
	PrecompileSet(Span),
	TestConcreteTypes(Span, Vec<syn::Type>),
}

impl syn::parse::Parse for ImplAttr {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		input.parse::<syn::Token![#]>()?;
		let content;
		syn::bracketed!(content in input);
		content.parse::<keyword::precompile>()?;
		content.parse::<syn::Token![::]>()?;

		let lookahead = content.lookahead1();

		if lookahead.peek(keyword::precompile_set) {
			Ok(ImplAttr::PrecompileSet(
				content.parse::<keyword::precompile_set>()?.span(),
			))
		} else if lookahead.peek(keyword::test_concrete_types) {
			let span = content.parse::<keyword::test_concrete_types>()?.span();

			let inner;
			syn::parenthesized!(inner in content);
			let types = inner.parse_terminated::<_, syn::Token![,]>(syn::Type::parse)?;

			Ok(ImplAttr::TestConcreteTypes(
				span,
				types.into_iter().collect(),
			))
		} else {
			Err(lookahead.error())
		}
	}
}
