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

pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	let ItemEnum {
		attrs,
		vis,
		enum_token,
		ident,
		variants,
		..
	} = item;

	let mut ident_expressions: Vec<Ident> = vec![];
	let mut variant_expressions: Vec<Expr> = vec![];
	let mut variant_attrs: Vec<Vec<Attribute>> = vec![];
	for variant in variants {
		match variant.discriminant {
			Some((_, Expr::Lit(ExprLit { lit, .. }))) => {
				if let Lit::Str(lit_str) = lit {
					let digest = Keccak256::digest(lit_str.value().as_bytes());
					let selector = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
					ident_expressions.push(variant.ident);
					variant_expressions.push(Expr::Lit(ExprLit {
						lit: Lit::Verbatim(Literal::u32_suffixed(selector)),
						attrs: Default::default(),
					}));
					variant_attrs.push(variant.attrs);
				} else {
					return quote_spanned! {
						lit.span() => compile_error!("Expected literal string");
					}
					.into();
				}
			}
			Some((_eg, expr)) => {
				return quote_spanned! {
					expr.span() => compile_error!("Expected literal");
				}
				.into()
			}
			None => {
				return quote_spanned! {
					variant.span() => compile_error!("Each variant must have a discriminant");
				}
				.into()
			}
		}
	}

	(quote! {
		#(#attrs)*
		#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
		#[repr(u32)]
		#vis #enum_token #ident {
			#(
				#(#variant_attrs)*
				#ident_expressions = #variant_expressions,
			)*
		}
	})
	.into()
}
