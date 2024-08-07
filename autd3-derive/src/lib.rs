mod builder;
mod gain;
mod modulation;

use proc_macro::TokenStream;

#[proc_macro_derive(Gain, attributes(no_gain_cache, no_gain_transform))]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    gain::impl_gain_macro(ast)
}

#[proc_macro_derive(
    Modulation,
    attributes(
        no_change,
        no_modulation_cache,
        no_modulation_transform,
        no_radiation_pressure
    )
)]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    modulation::impl_mod_macro(ast)
}

#[proc_macro_derive(Builder, attributes(no_const, get, get_mut, set))]
pub fn builder_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    builder::impl_builder_macro(ast)
}
