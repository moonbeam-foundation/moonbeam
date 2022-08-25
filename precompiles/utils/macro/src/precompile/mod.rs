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
use quote::{format_ident, quote, ToTokens};
use sha3::{Digest, Keccak256};
use std::collections::BTreeMap;
use syn::{parse_macro_input, spanned::Spanned};

pub mod attr;

pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
	let args = parse_macro_input!(args as syn::AttributeArgs);
	let mut impl_item = parse_macro_input!(item as syn::ItemImpl);

	let precompile = match Precompile::try_from(args, &mut impl_item) {
		Ok(p) => p,
		Err(e) => return e.into_compile_error().into(),
	};

	(quote! {
		#impl_item
	})
	.into()
}

pub struct Precompile {
	/// Impl struct type.
	struct_type: syn::Type,

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
	/// the Rust arguments. None if no arguments.
	///
	/// > EvmData trait allows to generate this string at runtime only. Thus
	/// > it is required to write it manually in the selector attribute, and
	/// > a unit test is generated to check it matches.
	solidity_arguments_type: Option<String>,

	/// Modifier of the function. They are all exclusive and defaults to
	/// `NonPayable`.
	modifier: Modifier,
}

#[derive(Debug)]
pub struct Argument {
	/// Identifier of the argument, which will be used in the struct variant.
	ident: syn::Ident,

	/// Type of the argument, which will be used in the struct variant and
	/// to parse the input.
	ty: syn::Type,
}

impl Precompile {
	pub fn try_from(mut args: syn::AttributeArgs, impl_: &mut syn::ItemImpl) -> syn::Result<Self> {
		println!(
			"Precompile: {}",
			impl_.self_ty.to_token_stream().to_string()
		);

		let enum_ident = Self::extract_enum_ident(args, impl_)?;

		println!("Enum: {}", enum_ident.to_token_stream().to_string());

		let mut precompile = Precompile {
			struct_type: impl_.self_ty.as_ref().clone(),
			enum_ident,
			generics: impl_.generics.clone(),
			selector_to_variant: BTreeMap::new(),
			variants_content: BTreeMap::new(),
			fallback_to_variant: None,
		};

		for mut item in &mut impl_.items {
			// We only interact with methods and leave the rest as-is.
			if let syn::ImplItem::Method(ref mut method) = &mut item {
				precompile.process_method(method)?;
			}
		}

		Ok(precompile)
	}

	fn extract_enum_ident(
		mut args: syn::AttributeArgs,
		impl_: &mut syn::ItemImpl,
	) -> syn::Result<syn::Ident> {
		let msg = "Macro expects the name of the enum that will be generated to parse the\
			call data. Please use `#[precompile(PrecompileInput)]`";

		if args.len() != 1 {
			return Err(syn::Error::new(Span::call_site(), msg));
		}

		let mut enum_path = match args.pop().expect("len checked above") {
			syn::NestedMeta::Meta(syn::Meta::Path(p)) => p,
			_ => return Err(syn::Error::new(Span::call_site(), msg)),
		};

		if let Some(colon) = enum_path.leading_colon {
			let msg = "Enum name must not have leading colon";
			return Err(syn::Error::new(colon.span(), msg));
		}

		if enum_path.segments.len() != 1 {
			let msg = "Enum name must must be a simple name without `::`";
			return Err(syn::Error::new(enum_path.segments.span(), msg));
		}

		let enum_path = enum_path
			.segments
			.pop()
			.expect("len checked above")
			.into_value();

		if enum_path.arguments != syn::PathArguments::None {
			let msg = format!(
				"Enum name must not have any arguments. Generics will automatically be\
			added to match those of {}",
				impl_.self_ty.to_token_stream().to_string()
			);
			return Err(syn::Error::new(enum_path.arguments.span(), msg));
		}

		Ok(enum_path.ident)
	}

	fn process_method(&mut self, method: &mut syn::ImplItemMethod) -> syn::Result<()> {
		// Take (remove) all attributes related to this macro.
		let attrs = attr::take_attributes::<attr::MethodAttr>(&mut method.attrs)?;

		// If there are no attributes it is a private function and we ignore it.
		if attrs.is_empty() {
			return Ok(());
		}

		// A method cannot have modifiers if it isn't a fallback and/or doesn't have a selector.
		let mut used = false;

		let method_name = method.sig.ident.clone();
		let mut modifier = Modifier::NonPayable;
		let mut solidity_arguments_type: Option<String> = None;
		let mut arguments = vec![];

		for attr in attrs {
			match attr {
				attr::MethodAttr::Fallback(span) => {
					if self.fallback_to_variant.is_some() {
						let msg = "A precompile can only have 1 fallback function";
						return Err(syn::Error::new(span, msg));
					}

					self.fallback_to_variant = Some(method_name.clone());
					used = true;
				}
				attr::MethodAttr::Payable(span) => {
					if modifier != Modifier::NonPayable {
						let msg =
							"A precompile method can have at most one modifier (payable, view)";
						return Err(syn::Error::new(span, msg));
					}

					modifier = Modifier::Payable;
				}
				attr::MethodAttr::View(span) => {
					if modifier != Modifier::NonPayable {
						let msg =
							"A precompile method can have at most one modifier (payable, view)";
						return Err(syn::Error::new(span, msg));
					}

					modifier = Modifier::View;
				}
				attr::MethodAttr::Public(span, signature_lit) => {
					used = true;

					let signature = signature_lit.value();

					// Split signature to get arguments type.
					let split: Vec<_> = signature.splitn(2, "(").collect();
					if split.len() != 2 {
						let msg = "Selector must have form \"foo(arg1,arg2,...)\"";
						return Err(syn::Error::new(signature_lit.span(), msg));
					}

					let local_args_type = format!("({}", split[1]); // add back initial parenthesis

					if let Some(ref args_type) = &solidity_arguments_type {
						// If there are multiple public attributes we check that they all have
						// the same type.
						if args_type != &local_args_type {
							let msg = "Method cannot have multiple selectors with different types.";
							return Err(syn::Error::new(signature_lit.span(), msg));
						}
					} else {
						solidity_arguments_type = Some(local_args_type);
					}

					// Compute the 4-bytes selector.
					let digest = Keccak256::digest(signature.as_ref());
					let selector = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);

					if let Some(previous) = self
						.selector_to_variant
						.insert(selector, method_name.clone())
					{
						let msg =
							format!("Selector collision with method {}", previous.to_string());
						return Err(syn::Error::new(signature_lit.span(), msg));
					}
				}
			}
		}

		if !used {
			let msg =
				"A precompile method cannot have modifiers without being a fallback or having\
			a selector";
			return Err(syn::Error::new(method.span(), msg));
		}

		// We forbid type parameters.
		if let Some(param) = method.sig.generics.params.first() {
			let msg = "Exposed precompile methods cannot have type parameters";
			return Err(syn::Error::new(param.span(), msg));
		}

		// We skip the first parameter which will be the PrecompileHandle.
		// Not having it or having a self parameter will produce a compilation error when
		// trying to call the functions with such PrecompileHandle.
		let method_inputs = method.sig.inputs.iter().skip(1);

		// We go through each parameter to collect each name and type that will be used to
		// generate the input enum and parse the call data.
		for input in method_inputs {
			let input = match input {
				syn::FnArg::Typed(t) => t,
				_ => {
					// I don't think it is possible to encounter this error since a self receiver
					// seems to only be possible in the first position which we skipped.
					let msg = "Exposed precompile methods cannot have a `self` parameter";
					return Err(syn::Error::new(input.span(), msg));
				}
			};

			let msg = "Parameter must be of the form `name: Type`, optionally prefixed by `mut`";
			let ident = match input.pat.as_ref() {
				syn::Pat::Ident(pat) => {
					if pat.by_ref.is_some() || pat.subpat.is_some() {
						return Err(syn::Error::new(pat.span(), msg));
					}

					pat.ident.clone()
				}
				_ => {
					return Err(syn::Error::new(input.pat.span(), msg));
				}
			};
			let ty = input.ty.as_ref().clone();

			arguments.push(Argument { ident, ty })
		}

		if let Some(_) = self.variants_content.insert(
			method_name.clone(),
			Variant {
				arguments,
				solidity_arguments_type,
				modifier,
			},
		) {
			let msg = "Duplicate method name";
			return Err(syn::Error::new(method_name.span(), msg));
		}

		Ok(())
	}

	// pub fn generate_enum(&self) -> impl ToTokens {

	// }
}
