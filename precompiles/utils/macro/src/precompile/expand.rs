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

use super::*;

impl Precompile {
	/// Main expand function, which expands everything else.
	pub fn expand(&self) -> impl ToTokens {
		let enum_ = self.expand_enum_decl();
		let enum_impl = self.expand_enum_impl();
		let precomp_impl = self.expand_precompile_impl();
		let test_signature = self.expand_test_solidity_signature();

		quote! {
			#enum_
			#enum_impl
			#precomp_impl
			#test_signature
		}
	}

	/// Expands the call enum declaration.
	pub fn expand_enum_decl(&self) -> impl ToTokens {
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

		quote!(
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
				__phantom(
					::core::marker::PhantomData<( #( #type_parameters ),* )>,
					::core::convert::Infallible
				),
			}
		)
	}

	/// Expands the parse function for each variants.
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

			quote!(
				use ::precompile_utils::solidity::modifier::FunctionModifier;
				use ::precompile_utils::evm::handle::PrecompileHandleExt;
				handle.check_function_modifier(FunctionModifier::#modifier)?;
			)
		});

		let variant_parsing = self
			.variants_content
			.iter()
			.map(|(variant_ident, variant)| {
				Self::expand_variant_parsing_from_handle(variant_ident, variant)
			});

		quote!(
			#(
				fn #fn_parse(
					handle: &mut impl PrecompileHandle
				) -> ::precompile_utils::EvmResult<Self> {
					use ::precompile_utils::solidity::revert::InjectBacktrace;

					#modifier_check
					#variant_parsing
				}
			)*
		)
	}

	/// Generates the parsing code for a variant, reading the input from the handle and
	/// parsing it using Reader.
	fn expand_variant_parsing_from_handle(
		variant_ident: &syn::Ident,
		variant: &Variant,
	) -> impl ToTokens {
		if variant.arguments.is_empty() {
			quote!( Ok(Self::#variant_ident {})).to_token_stream()
		} else {
			use case::CaseExt;

			let args_parse = variant.arguments.iter().map(|arg| {
				let ident = &arg.ident;
				let span = ident.span();
				let name = ident.to_string().to_camel_lowercase();

				quote_spanned!(span=> #ident: input.read().in_field(#name)?,)
			});
			let args_count = variant.arguments.len();

			quote!(
				let mut input = handle.read_after_selector()?;
				input.expect_arguments(#args_count)?;

				Ok(Self::#variant_ident {
					#(#args_parse)*
				})
			)
			.to_token_stream()
		}
	}

	/// Expands the call enum impl block.
	pub fn expand_enum_impl(&self) -> impl ToTokens {
		let enum_ident = &self.enum_ident;
		let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		let match_selectors = self.selector_to_variant.keys();
		let match_selectors2 = self.selector_to_variant.keys();

		let variants_parsing = self.expand_variants_parse_fn();

		let variants_ident2: Vec<_> = self.variants_content.keys().collect();
		let variants_selectors_fn: Vec<_> = self
			.variants_content
			.keys()
			.map(|name| format_ident!("{}_selectors", name))
			.collect();
		let variants_selectors: Vec<_> = self
			.variants_content
			.values()
			.map(|variant| &variant.selectors)
			.collect();

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

		quote!(
			impl #impl_generics #enum_ident #ty_generics #where_clause {
				#parse_call_data_fn

				#variants_parsing

				#execute_fn

				pub fn supports_selector(selector: u32) -> bool {
					match selector {
						#(
							#match_selectors => true,
						)*
						_ => false,
					}
				}

				pub fn selectors() -> &'static [u32] {
					&[#(
						#match_selectors2
					),*]
				}

				#(
					pub fn #variants_selectors_fn() -> &'static [u32] {
						&[#(
							#variants_selectors
						),*]
					}
				)*

				pub fn encode(self) -> ::sp_std::vec::Vec<u8> {
					use ::precompile_utils::solidity::codec::Writer;
					match self {
						#(
							Self::#variants_ident2 { #(#variants_list),* } => {
								#variants_encode
							},
						)*
						Self::__phantom(_, _) => panic!("__phantom variant should not be used"),
					}
				}
			}

			impl #impl_generics From<#enum_ident #ty_generics> for ::sp_std::vec::Vec<u8>
			#where_clause
			{
				fn from(a: #enum_ident #ty_generics) -> ::sp_std::vec::Vec<u8> {
					a.encode()
				}
			}
		)
	}

	/// Expand the execute fn of the enum.
	fn expand_enum_execute_fn(&self) -> impl ToTokens {
		let impl_type = &self.impl_type;

		let variants_ident: Vec<_> = self.variants_content.keys().collect();

		let variants_arguments: Vec<Vec<_>> = self
			.variants_content
			.values()
			.map(|variant| variant.arguments.iter().map(|arg| &arg.ident).collect())
			.collect();

		// If there is no precompile set there is no discriminant.
		let opt_discriminant_arg = self
			.precompile_set_discriminant_type
			.as_ref()
			.map(|ty| quote!( discriminant: #ty,));

		let variants_call = self
			.variants_content
			.iter()
			.map(|(variant_ident, variant)| {
				let arguments = variant.arguments.iter().map(|arg| &arg.ident);

				let output_span = variant.fn_output.span();
				let opt_discriminant_arg = self
					.precompile_set_discriminant_fn
					.as_ref()
					.map(|_| quote!(discriminant,));

				let write_output = quote_spanned!(output_span=>
					::precompile_utils::solidity::encode_return_value(output?)
				);

				quote!(
					let output = <#impl_type>::#variant_ident(
						#opt_discriminant_arg
						handle,
						#(#arguments),*
					);
					#write_output
				)
			});

		quote!(
			pub fn execute(
				self,
				#opt_discriminant_arg
				handle: &mut impl PrecompileHandle
			) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
				use ::precompile_utils::solidity::codec::Writer;
				use ::fp_evm::{PrecompileOutput, ExitSucceed};

				let output = match self {
					#(
						Self::#variants_ident { #(#variants_arguments),* } => {
							#variants_call
						},
					)*
					Self::__phantom(_, _) => panic!("__phantom variant should not be used"),
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
		match variant.selectors.first() {
			Some(selector) => {
				let write_arguments = variant.arguments.iter().map(|arg| {
					let ident = &arg.ident;
					let span = ident.span();
					quote_spanned!(span=> .write(#ident))
				});

				quote!(
					Writer::new_with_selector(#selector)
					#(#write_arguments)*
					.build()
				)
				.to_token_stream()
			}
			None => quote!(Default::default()).to_token_stream(),
		}
	}

	/// Expand the main parsing function that, based on the selector in the
	/// input, dispatch the decoding to one of the variants parsing function.
	fn expand_enum_parse_call_data(&self) -> impl ToTokens {
		let selectors = self.selector_to_variant.keys();
		let parse_fn = self
			.selector_to_variant
			.values()
			.map(Self::variant_ident_to_parse_fn);

		let match_fallback = match &self.fallback_to_variant {
			Some(variant) => {
				let parse_fn = Self::variant_ident_to_parse_fn(variant);
				quote!(_ => Self::#parse_fn(handle),).to_token_stream()
			}
			None => quote!(
				Some(_) => Err(RevertReason::UnknownSelector.into()),
				None => Err(RevertReason::read_out_of_bounds("selector").into()),
			)
			.to_token_stream(),
		};

		quote!(
			pub fn parse_call_data(
				handle: &mut impl PrecompileHandle
			) -> ::precompile_utils::EvmResult<Self> {
				use ::precompile_utils::solidity::revert::RevertReason;

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

	/// Expands the impl of the Precomile(Set) trait.
	pub fn expand_precompile_impl(&self) -> impl ToTokens {
		let impl_type = &self.impl_type;
		let enum_ident = &self.enum_ident;
		let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

		if let Some(discriminant_fn) = &self.precompile_set_discriminant_fn {
			let opt_pre_check = self.pre_check.as_ref().map(|ident| {
				let span = ident.span();
				quote_spanned!(span=>
					let _: () = <#impl_type>::#ident(discriminant, handle)
						.map_err(|err| Some(err))?;
				)
			});

			quote!(
				impl #impl_generics ::fp_evm::PrecompileSet for #impl_type #where_clause {
					fn execute(
						&self,
						handle: &mut impl PrecompileHandle
					) -> Option<::precompile_utils::EvmResult<::fp_evm::PrecompileOutput>> {
						use ::precompile_utils::precompile_set::DiscriminantResult;

						let discriminant = <#impl_type>::#discriminant_fn(
							handle.code_address(),
							handle.remaining_gas()
						);

						if let DiscriminantResult::Some(_, cost) | DiscriminantResult::None(cost) = discriminant {
							let result = handle.record_cost(cost);
							if let Err(e) = result {
								return Some(Err(e.into()));
							}
						}

						let discriminant = match discriminant {
							DiscriminantResult::Some(d, _) => d,
							DiscriminantResult::None(cost) => return None,
							DiscriminantResult::OutOfGas => return Some(Err(ExitError::OutOfGas.into()))
						};

						#opt_pre_check

						Some(
							<#enum_ident #ty_generics>::parse_call_data(handle)
								.and_then(|call| call.execute(discriminant, handle))
						)
					}

					fn is_precompile(&self, address: H160, gas: u64) -> ::fp_evm::IsPrecompileResult {
						<#impl_type>::#discriminant_fn(address, gas).into()
					}
				}
			)
			.to_token_stream()
		} else {
			let opt_pre_check = self.pre_check.as_ref().map(|ident| {
				let span = ident.span();
				quote_spanned!(span=>let _: () = <#impl_type>::#ident(handle)?;)
			});

			quote!(
				impl #impl_generics ::fp_evm::Precompile for #impl_type #where_clause {
					fn execute(
						handle: &mut impl PrecompileHandle
					) -> ::precompile_utils::EvmResult<::fp_evm::PrecompileOutput> {
						#opt_pre_check

						<#enum_ident #ty_generics>::parse_call_data(handle)?.execute(handle)
					}
				}
			)
			.to_token_stream()
		}
	}

	/// Expands the Solidity signature test.
	/// The macro expands an "inner" function in all build profiles, which is
	/// then called by a test in test profile. This allows to display errors that occurs in
	/// the expansion of the test without having to build in test profile, which is usually
	/// related to the use of a type parameter in one of the parsed parameters of a method.
	pub fn expand_test_solidity_signature(&self) -> impl ToTokens {
		let variant_test: Vec<_> = self
			.variants_content
			.iter()
			.map(|(ident, variant)| {
				let span = ident.span();

				let solidity = &variant.solidity_arguments_type;
				let name = ident.to_string();
				let types: Vec<_> = variant.arguments.iter().map(|arg| &arg.ty).collect();

				quote_spanned!(span=>
					assert_eq!(
						#solidity,
						<(#(#types,)*) as Codec>::signature(),
						"{} function signature doesn't match (left: attribute, right: computed \
						from Rust types)",
						#name
					);
				)
			})
			.collect();

		let test_name = format_ident!("__{}_test_solidity_signatures", self.impl_ident);
		let inner_name = format_ident!("__{}_test_solidity_signatures_inner", self.impl_ident);

		if let Some(test_types) = &self.test_concrete_types {
			let (impl_generics, _ty_generics, where_clause) = self.generics.split_for_impl();

			quote!(
				#[allow(non_snake_case)]
				pub(crate) fn #inner_name #impl_generics () #where_clause {
					use ::precompile_utils::solidity::Codec;
					#(#variant_test)*
				}

				#[test]
				#[allow(non_snake_case)]
				fn #test_name() {
					#inner_name::< #(#test_types),* >();
				}
			)
			.to_token_stream()
		} else {
			quote!(
				#[allow(non_snake_case)]
				pub(crate) fn #inner_name() {
					use ::precompile_utils::solidity::Codec;
					#(#variant_test)*
				}

				#[test]
				#[allow(non_snake_case)]
				fn #test_name() {
					#inner_name();
				}
			)
			.to_token_stream()
		}
	}
}
