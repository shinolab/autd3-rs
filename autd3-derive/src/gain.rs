use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> DatagramS for #name #ty_generics #where_clause
        {
            type G = GainOperationGenerator<<Self as Gain>::G>;
            type Error = GainError;

            fn operation_generator_with_segment(self, geometry: &Geometry, segment: Segment, transition_mode: Option<TransitionMode>) -> Result<Self::G, Self::Error> {
                Self::G::new(
                    self,
                    geometry,
                    segment,
                    transition_mode
                )
            }

            fn option(&self) -> DatagramOption {
                DatagramOption {
                    timeout: std::time::Duration::from_millis(20),
                    parallel_threshold: 4,
                }
            }
        }
    };

    let generator = quote! {
        #datagram
    };
    generator.into()
}
