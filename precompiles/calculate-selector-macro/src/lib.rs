#![crate_type = "proc-macro"]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use sha3::{Digest, Keccak256};
use syn::{parse_macro_input, Lit};

#[proc_macro]
pub fn calculate_fn_selector_for(input: TokenStream) -> TokenStream {
	let lit = parse_macro_input!(input as Lit);

	let expanded = if let Lit::Str(lit_str) = lit {
		let selector = &Keccak256::digest(lit_str.value().as_ref())[..4];
		let b1 = selector[0];
		let b2 = selector[1];
		let b3 = selector[2];
		let b4 = selector[3];
		quote! {
			[#b1, #b2, #b3, #b4]
		}
	} else {
		quote_spanned! {
			lit.span() => compile_error("expected literal string");
		}
	};

    TokenStream::from(expanded)
}
