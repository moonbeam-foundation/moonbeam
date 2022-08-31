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
	pub fn expand(&self) -> impl ToTokens {
		let enum_ = self.expand_enum_decl();
		let enum_impl = self.expand_enum_impl();
		let precomp_impl = self.expand_precompile_impl();

		quote! {
			#enum_
			#enum_impl
			#precomp_impl
		}
	}

	pub fn expand_enum_decl(&self) -> impl ToTokens {
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

	pub fn expand_variants_parse_fn(&self) -> impl ToTokens {
		let span = Span::call_site();

		let fn_parse = self
			.variants_content
			.keys()
			.map(Self::variant_ident_to_parse_fn);

		let modifier_check = self.variants_content.values().map(|variant| {
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

		let variant_parsing = self
			.variants_content
			.iter()
			.map(|(variant_ident, variant)| {
				Self::expand_variant_parsing_from_handle(variant_ident, variant)
			});

		quote_spanned!(span=>
			#(
				fn #fn_parse(handle: &mut impl PrecompileHandle) -> EvmResult<Self> {
					#modifier_check
					#variant_parsing
				}
			)*
		)
	}

	/// Generates the parsing code for a variant, reading the input from the handle and
	/// parsing it using EvmDataReader.
	fn expand_variant_parsing_from_handle(
		variant_ident: &syn::Ident,
		variant: &Variant,
	) -> impl ToTokens {
		let span = Span::call_site();
		if variant.arguments.is_empty() {
			quote_spanned!(span=> Ok(Self::#variant_ident {})).to_token_stream()
		} else {
			use case::CaseExt;

			let args_ident = variant.arguments.iter().map(|v| &v.ident);
			let args_name = variant
				.arguments
				.iter()
				.map(|v| v.ident.to_string().to_camel_lowercase());
			let args_count = variant.arguments.len();

			quote_spanned!(span=>
				let mut input = handle.read_after_selector()?;
				input.expect_arguments(#args_count)?;

				Ok(Self::#variant_ident {
					#(#args_ident: input.read().in_field(#args_name)?,)*
				})
			)
			.to_token_stream()
		}
	}

	pub fn expand_enum_impl(&self) -> impl ToTokens {
		let span = Span::call_site();
		let enum_ident = &self.enum_ident;
		let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		let match_selectors = self.selector_to_variant.keys();
		let match_selectors2 = self.selector_to_variant.keys();

		let variants_parsing = self.expand_variants_parse_fn();

		let variants_ident2: Vec<_> = self.variants_content.keys().collect();

		let variants_list: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|variant| variant.arguments.iter().map(|arg| &arg.ident).collect())
			.collect();

		let variants_encode: Vec<_> = self
			.variants_content
			.values()
			.map(Self::expand_variant_encoding)
			.collect();

		let parse_call_data_fn = self.expand_enum_parse_call_data();
		let execute_fn = self.expand_enum_execute_fn();
		let test_signature = self.expand_test_solidity_signature();

		quote_spanned!(span=>
			impl #impl_generics #enum_ident #ty_generics #where_clause {
				#parse_call_data_fn

				#variants_parsing

				#execute_fn

				#test_signature

				pub fn supports_selector(selector: u32) -> bool {
					match selector {
						#(
							#match_selectors => true,
						)*
						_ => false,
					}
				}

				pub fn selectors() -> impl Iterator<Item = u32> {
					vec![#(
						#match_selectors2
					),*].into_iter()
				}

				pub fn encode(self) -> Vec<u8> {
					match self {
						#(
							Self::#variants_ident2 { #(#variants_list),* } => {
								#variants_encode
							},
						)*
						Self::__phantom(_) => panic!("__phantom variant should not be used"),
					}
				}


			}

			#[cfg(test)]
			impl #impl_generics From<#enum_ident #ty_generics> for Vec<u8> #where_clause {
				fn from(a: #enum_ident #ty_generics) -> Vec<u8> {
					a.encode()
				}
			}
		)
	}

	/// Generate the execute fn of the enum.
	/// TODO: Support PrecompileSet which will require an additional argument.
	fn expand_enum_execute_fn(&self) -> impl ToTokens {
		let span = Span::call_site();

		let struct_type = &self.struct_type;

		let variants_ident: Vec<_> = self.variants_content.keys().collect();

		let variants_arguments: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|variant| variant.arguments.iter().map(|arg| &arg.ident).collect())
			.collect();

		// If there is no precompile set there is no discriminant.
		let opt_discriminant_arg = self.precompile_set_discriminant.as_ref().map(
			|PrecompileSetDiscriminant { type_, .. }| quote_spanned!(span=> discriminant: #type_,),
		);

		let variants_call = self
			.variants_content
			.iter()
			.map(|(variant_ident, variant)| {
				let arguments = variant.arguments.iter().map(|arg| &arg.ident);

				let opt_discriminant_arg = self
					.precompile_set_discriminant
					.as_ref()
					.map(|_| quote_spanned!(span=> discriminant,));

				quote_spanned!(span=>
					let output = <#struct_type>::#variant_ident(#opt_discriminant_arg handle, #(#arguments),*)?;
						EvmDataWriter::new().write(output).build()
				)
			});

		quote_spanned!(span=>
			pub fn execute(self, #opt_discriminant_arg handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
				use ::precompile_utils::data::EvmDataWriter;
				use ::fp_evm::{PrecompileOutput, ExitSucceed};

				let output = match self {
					#(
						Self::#variants_ident { #(#variants_arguments),* } => {
							#variants_call
						},
					)*
					Self::__phantom(_) => panic!("__phantom variant should not be used"),
				};

				Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output
				})
			}
		)
	}

	/// Expand how a variant can be Solidity encoded.
	fn expand_variant_encoding(variant: &Variant) -> impl ToTokens {
		let span = Span::call_site();
		match variant.encode_selector {
			Some(selector) => {
				let arguments = variant.arguments.iter().map(|arg| &arg.ident);

				quote_spanned!(span=>
					EvmDataWriter::new_with_selector(#selector)
					#(.write(#arguments))*
					.build()
				)
				.to_token_stream()
			}
			None => quote!(Vec::new()).to_token_stream(),
		}
	}

	/// Expand the main parsing function that, based on the selector in the
	/// input, dispatch the decoding to one of the variants parsing function.
	fn expand_enum_parse_call_data(&self) -> impl ToTokens {
		let span = Span::call_site();

		let selectors = self.selector_to_variant.keys();
		let parse_fn = self
			.selector_to_variant
			.values()
			.map(Self::variant_ident_to_parse_fn);

		let match_fallback = match &self.fallback_to_variant {
			Some(variant) => {
				let parse_fn = Self::variant_ident_to_parse_fn(variant);
				quote_spanned!(span=> _ => Self::#parse_fn(handle),).to_token_stream()
			}
			None => quote_spanned!(span=>
				Some(_) => Err(RevertReason::UnknownSelector.into()),
				None => Err(RevertReason::read_out_of_bounds("selector").into()),
			)
			.to_token_stream(),
		};

		quote_spanned!(span=>
			pub fn parse_call_data(handle: &mut impl PrecompileHandle) -> EvmResult<Self> {
				let input = handle.input();

				let selector = input.get(0..4).map(|s| {
					let mut buffer = [0u8; 4];
					buffer.copy_from_slice(s);
					u32::from_be_bytes(buffer)
				});

				match selector {
					#(
						Some(#selectors) => Self::#parse_fn(handle),
					)*
					#match_fallback
				}
			}
		)
	}

	fn variant_ident_to_parse_fn(ident: &syn::Ident) -> syn::Ident {
		format_ident!("_parse_{}", ident)
	}

	pub fn expand_precompile_impl(&self) -> impl ToTokens {
		let span = Span::call_site();
		let struct_type = &self.struct_type;
		let enum_ident = &self.enum_ident;
		let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		if let Some(PrecompileSetDiscriminant {
			fn_: discriminant_fn,
			..
		}) = &self.precompile_set_discriminant
		{
			quote_spanned!(span=>
				impl #impl_generics ::fp_evm::PrecompileSet for #struct_type #where_clause {
					fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
						let discriminant = match <#struct_type>::#discriminant_fn(handle.code_address()) {
							Some(d) => d,
							None => return None,
						};

						Some(
							<#enum_ident #ty_generics>::parse_call_data(handle)
								.and_then(|call| call.execute(discriminant, handle))
						)
					}

					fn is_precompile(&self, address: H160) -> bool {
						<#struct_type>::#discriminant_fn(address).is_some()
					}
				}
			)
			.to_token_stream()
		} else {
			quote_spanned!(span=>
				impl #impl_generics ::fp_evm::Precompile for #struct_type #where_clause {
					fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
						<#enum_ident #ty_generics>::parse_call_data(handle)?.execute(handle)
					}
				}
			)
			.to_token_stream()
		}
	}

	pub fn expand_test_solidity_signature(&self) -> impl ToTokens {
		let span = Span::call_site();

		let variant_name = self.variants_content.keys().map(|ident| ident.to_string());
		let variant_solidity = self
			.variants_content
			.values()
			.map(|v| &v.solidity_arguments_type);
		let variant_arguments_type: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|v| v.arguments.iter().map(|arg| &arg.ty).collect())
			.collect();

		quote_spanned!(span=>
			fn test_match_between_solidity_and_rust_types() {
				use ::precompile_utils::data::EvmData;
				#(
					assert_eq!(
						#variant_solidity,
						<( #(#variant_arguments_type,)* ) as EvmData>::solidity_type(),
						"{} function signature doesn't match (left: attribute, right: computed \
						from Rust types)",
						#variant_name
					);
				)*
			}
		)
	}
}
