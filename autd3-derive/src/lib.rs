#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! # A custom derive macro for `autd3`.

mod gain;
mod modulation;

use proc_macro::TokenStream;

#[proc_macro_derive(Gain)]
#[doc(hidden)]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    gain::impl_gain_macro(ast)
}

#[doc(hidden)]
#[proc_macro_derive(Modulation, attributes(manual_option))]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    modulation::impl_mod_macro(ast)
}
