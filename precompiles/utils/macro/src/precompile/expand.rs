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

use super::*;

impl Precompile {
	pub fn generate_enum(&self) -> impl ToTokens {
		let span = Span::call_site();
		let enum_ident = &self.enum_ident;
		let (_impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		let type_parameters = self.generics.type_params().map(|p| &p.ident);

		let variants: Vec<_> = self.variants_content.keys().collect();
		let idents: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|v| v.arguments.iter().map(|a| &a.ident).collect())
			.collect();
		let types: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|v| v.arguments.iter().map(|a| &a.ty).collect())
			.collect();

		quote_spanned!(span=>
			#[allow(non_camel_case_types)]
			pub enum #enum_ident #ty_generics #where_clause {
				#(
					#variants {
						#(
							#idents: #types
						),*
					},
				)*

				#[doc(hidden)]
				__phantom(PhantomData<( #( #type_parameters ),* )>),
			}
		)
	}

	pub fn generate_parse_functions(&self) -> impl ToTokens {
		let span = Span::call_site();

		let fn_parse: Vec<_> = self
			.variants_content
			.keys()
			.map(|ident| format_ident!("_parse_{}", ident))
			.collect();

		let variants_modifier_check = self.variants_content.values().map(|variant| {
			let modifier = match variant.modifier {
				Modifier::NonPayable => "NonPayable",
				Modifier::Payable => "Payable",
				Modifier::View => "View",
			};

			let modifier = syn::Ident::new(modifier, span);

			quote_spanned!(span=>
				use ::precompile_utils::modifier::FunctionModifier;
				handle.check_function_modifier(FunctionModifier::#modifier)?;
			)
		});

		let variants_parsing = self
			.variants_content
			.iter()
			.map(|(variant_ident, variant)| {
				if variant.arguments.is_empty() {
					quote!(Ok(Self::#variant_ident {})).to_token_stream()
				} else {
					use case::CaseExt;

					let args_ident = variant.arguments.iter().map(|v| &v.ident);
					let args_name = variant
						.arguments
						.iter()
						.map(|v| v.ident.to_string().to_camel_lowercase());
					let args_count = variant.arguments.len();

					quote!(
						let mut input = handle.read_after_selector()?;
						input.expect_arguments(#args_count)?;

						Ok(Self::#variant_ident {
							#(#args_ident: input.read().in_field(#args_name)?,)*
						})
					)
					.to_token_stream()
				}
			});

		quote_spanned!(span=>
			#(
				fn #fn_parse(handle: &mut impl PrecompileHandle) -> EvmResult<Self> {
					#variants_modifier_check
					#variants_parsing
				}
			)*
		)
	}

	pub fn generate_enum_impl(&self) -> impl ToTokens {
		let span = Span::call_site();
		let struct_type = &self.struct_type;
		let enum_ident = &self.enum_ident;
		let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		let match_selectors = self.selector_to_variant.keys();
		let match_variant_parse = self
			.selector_to_variant
			.values()
			.map(|variant| format_ident!("_parse_{}", variant));

		let match_fallback = match &self.fallback_to_variant {
			Some(variant) => {
				let variant_parse = format_ident!("_parse_{}", variant);
				quote!(Self::#variant_parse(handle)).to_token_stream()
			}
			None => quote!(Err(RevertReason::UnknownSelector.into())).to_token_stream(),
		};

		let variants_parsing = self.generate_parse_functions();

		let variants_ident: Vec<_> = self.variants_content.keys().map(|ident| ident).collect();

		let variants_list: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|variant| variant.arguments.iter().map(|arg| &arg.ident).collect())
			.collect();

		quote_spanned!(span=>
			impl #impl_generics #enum_ident #ty_generics #where_clause {
				pub fn parse_call_data(handle: &mut impl PrecompileHandle) -> EvmResult<Self> {
					let input = handle.input();

					if input.len() < 4 {
						return Err(RevertReason::read_out_of_bounds("selector").into());
					}

					let mut buffer = [0u8; 4];
					buffer.copy_from_slice(&input[0..4]);
					let selector = u32::from_be_bytes(buffer);

					match selector {
						#(
							#match_selectors => Self::#match_variant_parse(handle),
						)*
						_ => #match_fallback,
					}
				}

				#variants_parsing

				pub fn execute(self, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
					use ::precompile_utils::data::EvmDataWriter;
					use ::fp_evm::{PrecompileOutput, ExitSucceed};

					let output = match self {
						#(
							Self::#variants_ident { #(#variants_list),* } => {
								let output = <#struct_type>::#variants_ident(handle, #(#variants_list),*)?;
								EvmDataWriter::new().write(output).build()
							},
						)*
						Self::__phantom(_) => panic!("__phantom variant should not be used"),
					};

					Ok(
						PrecompileOutput {
							exit_status: ExitSucceed::Returned,
							output
						}
					)
				}
			}
		)
	}

	pub fn generate_precompile_impl(&self) -> impl ToTokens {
		let span = Span::call_site();
		let struct_type = &self.struct_type;
		let enum_ident = &self.enum_ident;
		let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		quote_spanned!(span=>
			impl #impl_generics ::fp_evm::Precompile for #struct_type #where_clause {
				fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
					<#enum_ident #ty_generics>::parse_call_data(handle)?.execute(handle)
				}
			}
		)
	}
}
