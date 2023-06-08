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
	/// Try to extract information out of an annotated `impl` block.
	pub fn try_from(impl_: &mut syn::ItemImpl) -> syn::Result<Self> {
		// Extract the name of the type used in the `impl` block.
		let impl_ident = Self::extract_impl_ident(impl_)?;
		let enum_ident = format_ident!("{}Call", impl_ident);

		// We setup the data collection struct.
		let mut precompile = Precompile {
			impl_type: impl_.self_ty.as_ref().clone(),
			impl_ident,
			enum_ident,
			generics: impl_.generics.clone(),
			selector_to_variant: BTreeMap::new(),
			variants_content: BTreeMap::new(),
			fallback_to_variant: None,
			tagged_as_precompile_set: false,
			precompile_set_discriminant_fn: None,
			precompile_set_discriminant_type: None,
			test_concrete_types: None,
			pre_check: None,
		};

		precompile.process_impl_attr(impl_)?;
		for mut item in &mut impl_.items {
			// We only interact with methods and leave the rest as-is.
			if let syn::ImplItem::Method(ref mut method) = &mut item {
				precompile.process_method(method)?;
			}
		}

		// Check constraint of PrecompileSet.
		if precompile.tagged_as_precompile_set
			&& precompile.precompile_set_discriminant_fn.is_none()
		{
			let msg = "A PrecompileSet must have exactly one function tagged with \
			`#[precompile::discriminant]`";
			return Err(syn::Error::new(Span::call_site(), msg));
		}

		Ok(precompile)
	}

	/// Process the attributes used on the `impl` block, which allows to declare
	/// if it is a PrecompileSet or not, and to provide concrete types for tests if necessary.
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

	/// Extract the ident of the type of the `impl` block.
	/// This ident is used to generate new idents such as the name of the Call enum and
	/// the Solidity selector test.
	fn extract_impl_ident(impl_: &syn::ItemImpl) -> syn::Result<syn::Ident> {
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

	/// Process a single method, looking for attributes and checking mandatory parameters.
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

		// We first look for unique attributes.
		if let Some(attr::MethodAttr::Discriminant(span)) = attrs.first() {
			let span = *span;

			if attrs.len() != 1 {
				let msg = "The discriminant attribute must be the only precompile attribute of \
				a function";
				return Err(syn::Error::new(span, msg));
			}

			return self.parse_discriminant_fn(span, method);
		}

		if let Some(attr::MethodAttr::PreCheck(span)) = attrs.first() {
			let span = *span;

			if attrs.len() != 1 {
				let msg = "The pre_check attribute must be the only precompile attribute of \
				a function";
				return Err(syn::Error::new(span, msg));
			}

			return self.parse_pre_check_fn(span, method);
		}

		// We iterate over all attributes of the method.
		for attr in attrs {
			match attr {
				attr::MethodAttr::Discriminant(span) => {
					let msg = "The discriminant attribute must be the only precompile \
					attribute of the function";
					return Err(syn::Error::new(span, msg));
				}
				attr::MethodAttr::PreCheck(span) => {
					let msg = "The pre_check attribute must be the only precompile \
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
				attr::MethodAttr::Public(_, signature_lit) => {
					used = true;

					let selector = self.parse_public_attr(
						signature_lit,
						&method_name,
						&mut solidity_arguments_type,
					)?;
					selectors.push(selector);
				}
			}
		}

		// A method cannot have attributes without being public or fallback.
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

		// Fallback method cannot have custom parameters.
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

		let mut method_inputs = method.sig.inputs.iter();

		// We check the first parameters of the method.
		// If this is a PrecompileSet it will look for a discriminant.
		// Then for all precompile(set)s it will look for the PrecompileHandle.
		// We take them from the iterator such that we are only left with the
		// custom arguments.
		self.check_initial_parameters(&mut method_inputs, method.sig.span())?;

		// We go through each parameter to collect each name and type that will be used to
		// generate the input enum and parse the call data.
		for input in method_inputs {
			let input = match input {
				syn::FnArg::Typed(t) => t,
				_ => {
					// I don't think it is possible to encounter this error since a self receiver
					// seems to only be possible in the first position which is checked in
					// `check_initial_parameters`.
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
			self.check_type_parameter_usage(&ty)?;

			arguments.push(Argument { ident, ty })
		}

		// Function output.
		let output_type = match &method.sig.output {
			syn::ReturnType::Type(_, t) => t,
			_ => {
				let msg = "A precompile method must have a return type of `EvmResult<_>` (exposed \
				by `precompile_utils`)";
				return Err(syn::Error::new(method.sig.span(), msg));
			}
		};

		// We insert the collected data in self.
		if let Some(_) = self.variants_content.insert(
			method_name.clone(),
			Variant {
				arguments,
				solidity_arguments_type: solidity_arguments_type.unwrap_or(String::from("()")),
				modifier,
				selectors,
				fn_output: output_type.as_ref().clone(),
			},
		) {
			let msg = "Duplicate method name";
			return Err(syn::Error::new(method_name.span(), msg));
		}

		Ok(())
	}

	/// Check the initial parameters of most methods of a Precompile(Set).
	fn check_initial_parameters<'a>(
		&mut self,
		method_inputs: &mut impl Iterator<Item = &'a syn::FnArg>,
		method_span: Span,
	) -> syn::Result<()> {
		// Discriminant input
		if self.tagged_as_precompile_set {
			let input = match method_inputs.next() {
				Some(a) => a,
				None => {
					let msg = "PrecompileSet methods must have at least 2 parameters (the \
					precompile instance discriminant and the PrecompileHandle)";
					return Err(syn::Error::new(method_span, msg));
				}
			};

			let input = match input {
				syn::FnArg::Typed(a) => a,
				_ => {
					let msg = "self is not allowed in precompile methods";
					return Err(syn::Error::new(input.span(), msg));
				}
			};

			let input_type = input.ty.as_ref();

			self.try_register_discriminant_type(&input_type)?;
		}

		// Precompile handle input
		{
			let input = match method_inputs.next() {
				Some(a) => a,
				None => {
					let msg = if self.tagged_as_precompile_set {
						"PrecompileSet methods must have at least 2 parameters (the precompile \
						instance discriminant and the PrecompileHandle)"
					} else {
						"Precompile methods must have at least 1 parameter (the PrecompileHandle)"
					};

					return Err(syn::Error::new(method_span, msg));
				}
			};

			let input = match input {
				syn::FnArg::Typed(a) => a,
				_ => {
					let msg = "self is not allowed in precompile methods";
					return Err(syn::Error::new(input.span(), msg));
				}
			};

			let input_type = input.ty.as_ref();

			if !is_same_type(&input_type, &syn::parse_quote! {&mut impl PrecompileHandle}) {
				let msg = "This parameter must have type `&mut impl PrecompileHandle`";
				return Err(syn::Error::new(input_type.span(), msg));
			}
		}

		Ok(())
	}

	/// Records the type of the discriminant and ensure they all have the same type.
	fn try_register_discriminant_type(&mut self, ty: &syn::Type) -> syn::Result<()> {
		if let Some(known_type) = &self.precompile_set_discriminant_type {
			if !is_same_type(&known_type, &ty) {
				let msg = format!(
					"All discriminants must have the same type (found {} before)",
					known_type.to_token_stream()
				);
				return Err(syn::Error::new(ty.span(), msg));
			}
		} else {
			self.precompile_set_discriminant_type = Some(ty.clone());
		}

		Ok(())
	}

	/// Process the discriminant function.
	fn parse_discriminant_fn(
		&mut self,
		span: Span,
		method: &syn::ImplItemMethod,
	) -> syn::Result<()> {
		if !self.tagged_as_precompile_set {
			let msg = "The impl block must be tagged with `#[precompile::precompile_set]` for
			the discriminant attribute to be used";
			return Err(syn::Error::new(span, msg));
		}

		if self.precompile_set_discriminant_fn.is_some() {
			let msg = "A PrecompileSet can only have 1 discriminant function";
			return Err(syn::Error::new(span, msg));
		}

		let span = method.sig.span();

		if method.sig.inputs.len() != 2 {
			let msg = "The discriminant function must only take code address (H160) and \
			remaining gas (u64) as parameters.";
			return Err(syn::Error::new(span, msg));
		}

		let msg = "The discriminant function must return an DiscriminantResult<_> (no type alias)";

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

		if return_segment.ident.to_string() != "DiscriminantResult" {
			return Err(syn::Error::new(return_segment.ident.span(), msg));
		}

		let result_arguments = match &return_segment.arguments {
			syn::PathArguments::AngleBracketed(args) => args,
			_ => return Err(syn::Error::new(return_segment.ident.span(), msg)),
		};

		if result_arguments.args.len() != 1 {
			let msg = "DiscriminantResult type should only have 1 type argument";
			return Err(syn::Error::new(result_arguments.args.span(), msg));
		}

		let discriminant_type: &syn::Type = match &result_arguments.args[0] {
			syn::GenericArgument::Type(t) => t,
			_ => return Err(syn::Error::new(result_arguments.args.span(), msg)),
		};

		self.try_register_discriminant_type(&discriminant_type)?;

		self.precompile_set_discriminant_fn = Some(method.sig.ident.clone());

		Ok(())
	}

	/// Process the pre_check function.
	fn parse_pre_check_fn(&mut self, span: Span, method: &syn::ImplItemMethod) -> syn::Result<()> {
		if self.pre_check.is_some() {
			let msg = "A Precompile can only have 1 pre_check function";
			return Err(syn::Error::new(span, msg));
		}

		let span = method.sig.span();

		let mut method_inputs = method.sig.inputs.iter();

		self.check_initial_parameters(&mut method_inputs, span)?;

		if method_inputs.next().is_some() {
			let msg = if self.tagged_as_precompile_set {
				"PrecompileSet pre_check method must have exactly 2 parameters (the precompile \
				instance discriminant and the PrecompileHandle)"
			} else {
				"Precompile pre_check method must have exactly 1 parameter (the \
				PrecompileHandle)"
			};

			return Err(syn::Error::new(span, msg));
		}

		self.pre_check = Some(method.sig.ident.clone());

		Ok(())
	}

	/// Process a `public` attribute on a method.
	fn parse_public_attr(
		&mut self,
		signature_lit: syn::LitStr,
		method_name: &syn::Ident,
		solidity_arguments_type: &mut Option<String>,
	) -> syn::Result<u32> {
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
		if let Some(ref args_type) = solidity_arguments_type {
			if args_type != &local_args_type {
				let msg = "Method cannot have selectors with different types.";
				return Err(syn::Error::new(signature_lit.span(), msg));
			}
		} else {
			*solidity_arguments_type = Some(local_args_type);
		}

		// Compute the 4-bytes selector.
		let digest = Keccak256::digest(signature.as_bytes());
		let selector = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);

		if let Some(previous) = self
			.selector_to_variant
			.insert(selector, method_name.clone())
		{
			let msg = format!("Selector collision with method {}", previous.to_string());
			return Err(syn::Error::new(signature_lit.span(), msg));
		}

		Ok(selector)
	}

	/// Check that the provided type doesn't depend on one of the type parameters of the
	/// precompile. Check is skipped if `test_concrete_types` attribute is used.
	fn check_type_parameter_usage(&self, ty: &syn::Type) -> syn::Result<()> {
		if self.test_concrete_types.is_some() {
			return Ok(());
		}

		const ERR_MESSAGE: &'static str =
			"impl type parameter is used in functions arguments. Arguments should not have a type
depending on a type parameter, unless it is a length bound for BoundedBytes,
BoundedString or alike, which doesn't affect the Solidity type.

In that case, you must add a #[precompile::test_concrete_types(...)] attribute on the impl
block to provide concrete types that will be used to run the automatically generated tests
ensuring the Solidity function signatures are correct.";

		match ty {
			syn::Type::Array(syn::TypeArray { elem, .. })
			| syn::Type::Group(syn::TypeGroup { elem, .. })
			| syn::Type::Paren(syn::TypeParen { elem, .. })
			| syn::Type::Reference(syn::TypeReference { elem, .. })
			| syn::Type::Ptr(syn::TypePtr { elem, .. })
			| syn::Type::Slice(syn::TypeSlice { elem, .. }) => self.check_type_parameter_usage(&elem)?,

			syn::Type::Path(syn::TypePath {
				path: syn::Path { segments, .. },
				..
			}) => {
				let impl_params: Vec<_> = self
					.generics
					.params
					.iter()
					.filter_map(|param| match param {
						syn::GenericParam::Type(syn::TypeParam { ident, .. }) => Some(ident),
						_ => None,
					})
					.collect();

				for segment in segments {
					if impl_params.contains(&&segment.ident) {
						return Err(syn::Error::new(segment.ident.span(), ERR_MESSAGE));
					}

					if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
						let types = args.args.iter().filter_map(|arg| match arg {
							syn::GenericArgument::Type(ty)
							| syn::GenericArgument::Binding(syn::Binding { ty, .. }) => Some(ty),
							_ => None,
						});

						for ty in types {
							self.check_type_parameter_usage(&ty)?;
						}
					}
				}
			}
			syn::Type::Tuple(tuple) => {
				for ty in tuple.elems.iter() {
					self.check_type_parameter_usage(ty)?;
				}
			}
			// BareFn => very unlikely this appear as parameter
			// ImplTrait => will cause other errors, it must be a concrete type
			// TypeInfer => it must be explicit concrete types since it ends up in enum fields
			// Macro => Cannot check easily
			// Never => Function will not be callable.
			ty => println!("Skipping type parameter check for non supported kind of type: {ty:?}"),
		}

		Ok(())
	}
}

/// Helper to check 2 types are equal.
/// Having a function with explicit type annotation helps type inference at callsite,
/// which have trouble if `==` is used inline.
fn is_same_type(a: &syn::Type, b: &syn::Type) -> bool {
	a == b
}
