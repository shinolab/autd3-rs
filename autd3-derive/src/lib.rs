#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! # A custom derive macro for `autd3`.

mod gain;
mod modulation;
mod parser;

use proc_macro::TokenStream;

#[proc_macro_derive(Gain)]
#[doc(hidden)]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let parsed = parser::parse_derive_input(input);
    gain::impl_gain_macro(parsed)
}

#[doc(hidden)]
#[proc_macro_derive(Modulation, attributes(manual_option))]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let parsed = parser::parse_derive_input(input);
    modulation::impl_mod_macro(parsed)
}
