mod builder;
mod gain;
mod modulation;

use proc_macro::TokenStream;

#[proc_macro_derive(Gain)]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    gain::impl_gain_macro(ast)
}

#[proc_macro_derive(Modulation, attributes(no_change))]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    modulation::impl_mod_macro(ast)
}

#[proc_macro_derive(Builder, attributes(get, set, as_bytes))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    builder::impl_builder_macro(ast)
}
