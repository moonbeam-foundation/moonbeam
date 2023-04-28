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
use syn::{GenericArgument, Type};

pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemType);

	let ItemType {
		attrs,
		vis,
		type_token,
		ident,
		generics,
		eq_token,
		ty,
		semi_token,
	} = item;

	if let Type::Tuple(ref type_tuple) = *ty {
		let variants: Vec<(Ident, u64)> = type_tuple
			.elems
			.iter()
			.filter_map(extract_precompile_name_and_prefix)
			.collect();

		let ident_expressions: Vec<&Ident> = variants.iter().map(|(ident, _)| ident).collect();
		let variant_expressions: Vec<&u64> = variants.iter().map(|(_, id)| id).collect();

		(quote! {
			#(#attrs)*
			#vis #type_token #ident #generics #eq_token #ty #semi_token

			#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
			#[repr(u64)]
			pub enum PrecompileName {
				#(
					#ident_expressions = #variant_expressions,
				)*
			}

			impl PrecompileName {
				pub fn from_address(address: sp_core::H160) -> Option<Self> {
					let _u64 = address.to_low_u64_be();
					if address == sp_core::H160::from_low_u64_be(_u64) {
						use num_enum::TryFromPrimitive;
						Self::try_from_primitive(_u64).ok()
					} else {
						None
					}
				}
			}
		})
		.into()
	} else {
		return quote_spanned! {
			ty.span() => compile_error!("Expected tuple");
		}
		.into();
	}
}

fn extract_precompile_name_and_prefix(type_: &Type) -> Option<(Ident, u64)> {
	match type_ {
		Type::Path(type_path) => {
			if let Some(path_segment) = type_path.path.segments.last() {
				match path_segment.ident.to_string().as_ref() {
					"PrecompileAt" => {
						extract_precompile_name_and_prefix_for_precompile_at(path_segment)
					}
					_ => None,
				}
			} else {
				None
			}
		}
		_ => None,
	}
}

fn extract_precompile_name_and_prefix_for_precompile_at(
	path_segment: &syn::PathSegment,
) -> Option<(Ident, u64)> {
	if let syn::PathArguments::AngleBracketed(generics) = &path_segment.arguments {
		let mut iter = generics.args.iter();
		if let (
			Some(GenericArgument::Type(Type::Path(type_path_1))),
			Some(GenericArgument::Type(Type::Path(type_path_2))),
		) = (iter.next(), iter.next())
		{
			if let (Some(path_segment_1), Some(path_segment_2)) = (
				type_path_1.path.segments.last(),
				type_path_2.path.segments.last(),
			) {
				if let syn::PathArguments::AngleBracketed(generics_) = &path_segment_1.arguments {
					if let Some(GenericArgument::Const(Expr::Lit(lit))) = generics_.args.first() {
						if let Lit::Int(int) = &lit.lit {
							if let Ok(precompile_id) = int.base10_parse() {
								if &path_segment_2.ident.to_string() == "CollectivePrecompile" {
									if let Some(instance_ident) =
										precompile_instance_ident(&path_segment_2)
									{
										return Some((instance_ident, precompile_id));
									}
								} else {
									return Some((path_segment_2.ident.clone(), precompile_id));
								}
							}
						}
					}
				}
			}
		}
	}

	None
}

fn precompile_instance_ident(path_segment: &syn::PathSegment) -> Option<Ident> {
	if let syn::PathArguments::AngleBracketed(generics_) = &path_segment.arguments {
		if let Some(GenericArgument::Type(Type::Path(instance_type_path))) = generics_.args.last() {
			if let Some(instance_type) = instance_type_path.path.segments.last() {
				return Some(instance_type.ident.clone());
			}
		}
	}

	None
}
