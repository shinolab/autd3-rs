#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! # A custom derive macro for `autd3`.

mod builder;
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
#[proc_macro_derive(Modulation, attributes(no_change))]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    modulation::impl_mod_macro(ast)
}

#[proc_macro_derive(Builder, attributes(get, set))]
#[doc(hidden)]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    builder::impl_builder_macro(ast)
}
