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
	pub fn try_from(_args: syn::AttributeArgs, impl_: &mut syn::ItemImpl) -> syn::Result<Self> {
		let struct_ident = Self::extract_struct_ident(impl_)?;
		let enum_ident = format_ident!("{}Call", struct_ident);

		let mut precompile = Precompile {
			struct_type: impl_.self_ty.as_ref().clone(),
			struct_ident,
			enum_ident,
			generics: impl_.generics.clone(),
			selector_to_variant: BTreeMap::new(),
			variants_content: BTreeMap::new(),
			fallback_to_variant: None,
			tagged_as_precompile_set: false,
			precompile_set_discriminant: None,
			test_concrete_types: None,
			pre_dispatch_check: None,
		};

		precompile.process_impl_attr(impl_)?;

		for mut item in &mut impl_.items {
			// We only interact with methods and leave the rest as-is.
			if let syn::ImplItem::Method(ref mut method) = &mut item {
				precompile.process_method(method)?;
			}
		}

		if precompile.tagged_as_precompile_set && precompile.precompile_set_discriminant.is_none() {
			let msg = "A PrecompileSet must have exactly one function tagged with \
			`#[precompile::discriminant]`";
			return Err(syn::Error::new(impl_.span(), msg));
		}

		Ok(precompile)
	}

	fn process_impl_attr(&mut self, impl_: &mut syn::ItemImpl) -> syn::Result<()> {
		let attrs = attr::take_attributes::<attr::ImplAttr>(&mut impl_.attrs)?;

		for attr in attrs {
			match attr {
				attr::ImplAttr::PrecompileSet(_) => {
					self.tagged_as_precompile_set = true;
				}
				attr::ImplAttr::TestConcreteTypes(span, types) => {
					if types.len() != self.generics.params.len() {
						let msg = "The amount of types should match the amount of type parameters \
						of the impl block";
						return Err(syn::Error::new(span, msg));
					}

					if self.test_concrete_types.is_some() {
						let msg = "Only one set of types can be provided to generate tests";
						return Err(syn::Error::new(span, msg));
					}

					self.test_concrete_types = Some(types);
				}
			}
		}

		Ok(())
	}

	fn extract_struct_ident(impl_: &syn::ItemImpl) -> syn::Result<syn::Ident> {
		let type_path = match impl_.self_ty.as_ref() {
			syn::Type::Path(p) => p,
			_ => {
				let msg = "The type in the impl block must be a path, like `Precompile` or
				`example::Precompile`";
				return Err(syn::Error::new(impl_.self_ty.span(), msg));
			}
		};

		let final_path = type_path.path.segments.last().ok_or_else(|| {
			let msg = "The type path must be non empty.";
			syn::Error::new(impl_.self_ty.span(), msg)
		})?;

		Ok(final_path.ident.clone())
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
		let mut is_fallback = false;
		let mut selectors = vec![];
		let initial_arguments = if self.tagged_as_precompile_set { 2 } else { 1 };

		if let Some(attr::MethodAttr::Discriminant(span)) = attrs.first() {
			let span = *span;
			if !self.tagged_as_precompile_set {
				let msg = "The impl block must be tagged with `#[precompile::precompile_set]` for
				the discriminant attribute to be used";
				return Err(syn::Error::new(span, msg));
			}

			if self.precompile_set_discriminant.is_some() {
				let msg = "A PrecompileSet can only have 1 discriminant function";
				return Err(syn::Error::new(span, msg));
			}

			if attrs.len() != 1 {
				let msg = "The discriminant attribute must be the only precompile attribute of \
				a function";
				return Err(syn::Error::new(span, msg));
			}

			let span = method.span();

			if method.sig.inputs.len() != 1 {
				let msg = "The discriminant function must only take the code address as parameter.";
				return Err(syn::Error::new(span, msg));
			}

			let msg = "The discriminant function must return an Option of the discriminant type";

			let return_type = match &method.sig.output {
				syn::ReturnType::Type(_, t) => t.as_ref(),
				_ => return Err(syn::Error::new(span, msg)),
			};

			let return_path = match return_type {
				syn::Type::Path(p) => p,
				_ => return Err(syn::Error::new(span, msg)),
			};

			if return_path.qself.is_some() {
				return Err(syn::Error::new(span, msg));
			}

			let return_path = &return_path.path;

			if return_path.leading_colon.is_some() || return_path.segments.len() != 1 {
				return Err(syn::Error::new(span, msg));
			}

			let return_segment = &return_path.segments[0];

			if return_segment.ident.to_string() != "Option" {
				return Err(syn::Error::new(return_segment.ident.span(), msg));
			}

			let option_arguments = match &return_segment.arguments {
				syn::PathArguments::AngleBracketed(args) => args,
				_ => return Err(syn::Error::new(return_segment.ident.span(), msg)),
			};

			if option_arguments.args.len() != 1 {
				let msg = "Option type should only have 1 type argument";
				return Err(syn::Error::new(option_arguments.args.span(), msg));
			}

			let discriminant_type = match &option_arguments.args[0] {
				syn::GenericArgument::Type(t) => t,
				_ => return Err(syn::Error::new(option_arguments.args.span(), msg)),
			};

			self.precompile_set_discriminant = Some(PrecompileSetDiscriminant {
				fn_: method.sig.ident.clone(),
				type_: discriminant_type.clone(),
			});

			return Ok(());
		}

		if let Some(attr::MethodAttr::PreDispatchCheck(span)) = attrs.first() {
			let span = *span;

			if self.pre_dispatch_check.is_some() {
				let msg = "A Precompile can only have 1 pre_dispatch_check function";
				return Err(syn::Error::new(span, msg));
			}

			if attrs.len() != 1 {
				let msg =
					"The pre_dispatch_check attribute must be the only precompile attribute of \
				a function";
				return Err(syn::Error::new(span, msg));
			}

			let span = method.span();

			if method.sig.inputs.len() != initial_arguments {
				let msg = if self.tagged_as_precompile_set {
					"PrecompileSet pre_dispatch_check method must have exactly 2 parameters (the precompile instance \
					discriminant and the PrecompileHandle)"
				} else {
					"Precompile pre_dispatch_check method must have exactly 1 parameter (the PrecompileHandle)"
				};

				return Err(syn::Error::new(span, msg));
			}

			self.pre_dispatch_check = Some(method.sig.ident.clone());

			return Ok(());
		}

		for attr in attrs {
			match attr {
				attr::MethodAttr::Discriminant(span) => {
					let msg = "The discriminant attribute must be the only precompile \
					attribute of the function";
					return Err(syn::Error::new(span, msg));
				}
				attr::MethodAttr::PreDispatchCheck(span) => {
					let msg = "The pre_dispatch_check attribute must be the only precompile \
					attribute of the function";
					return Err(syn::Error::new(span, msg));
				}
				attr::MethodAttr::Fallback(span) => {
					if self.fallback_to_variant.is_some() {
						let msg = "A precompile can only have 1 fallback function";
						return Err(syn::Error::new(span, msg));
					}

					self.fallback_to_variant = Some(method_name.clone());
					used = true;
					is_fallback = true;
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
				attr::MethodAttr::Public(_span, signature_lit) => {
					used = true;

					let signature = signature_lit.value();

					// Split signature to get arguments type.
					let split: Vec<_> = signature.splitn(2, "(").collect();
					if split.len() != 2 {
						let msg = "Selector must have form \"foo(arg1,arg2,...)\"";
						return Err(syn::Error::new(signature_lit.span(), msg));
					}

					let local_args_type = format!("({}", split[1]); // add back initial parenthesis

					// If there are multiple public attributes we check that they all have
					// the same type.
					if let Some(ref args_type) = &solidity_arguments_type {
						if args_type != &local_args_type {
							let msg = "Method cannot have selectors with different types.";
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

					selectors.push(selector);
				}
			}
		}

		if !used {
			let msg =
				"A precompile method cannot have modifiers without being a fallback or having\
			a `public` attribute";
			return Err(syn::Error::new(method.span(), msg));
		}

		// We forbid type parameters.
		if let Some(param) = method.sig.generics.params.first() {
			let msg = "Exposed precompile methods cannot have type parameters";
			return Err(syn::Error::new(param.span(), msg));
		}

		if method.sig.inputs.len() < initial_arguments {
			let msg = if self.tagged_as_precompile_set {
				"PrecompileSet methods must have at least 2 parameters (the precompile instance \
				discriminant and the PrecompileHandle)"
			} else {
				"Precompile methods must have at least 1 parameter (the PrecompileHandle)"
			};

			return Err(syn::Error::new(method.span(), msg));
		}

		if is_fallback {
			if let Some(input) = method.sig.inputs.iter().skip(initial_arguments).next() {
				let msg = if self.tagged_as_precompile_set {
					"Fallback methods cannot take any parameter outside of the discriminant and \
					PrecompileHandle"
				} else {
					"Fallback methods cannot take any parameter outside of the PrecompileHandle"
				};

				return Err(syn::Error::new(input.span(), msg));
			}
		}

		// We skip the initial parameter(s).
		// Not having it/them or having a self parameter will produce a compilation error when
		// trying to call the functions.
		let method_inputs = method.sig.inputs.iter().skip(initial_arguments);

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
				solidity_arguments_type: solidity_arguments_type.unwrap_or(String::from("()")),
				modifier,
				selectors,
			},
		) {
			let msg = "Duplicate method name";
			return Err(syn::Error::new(method_name.span(), msg));
		}

		Ok(())
	}
}
