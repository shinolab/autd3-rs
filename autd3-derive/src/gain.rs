use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram = quote! {
        impl <#(#linetimes,)* #(#type_params,)*> DatagramS for #name #ty_generics #where_clause
        {
            type G =  GainOperationGenerator<<Self as Gain>::G>;

            fn operation_generator_with_segment(self, geometry: &Geometry, segment: Segment, transition: Option<TransitionMode>) -> Result<Self::G, AUTDInternalError> {
                Self::G::new(
                    self,
                    geometry,
                    segment,
                    transition,
                )
            }

            fn timeout(&self) -> Option<std::time::Duration> {
                None
            }

            fn parallel_threshold(&self) -> Option<usize> {
                None
            }
        }
    };

    let generator = quote! {
        #datagram
    };
    generator.into()
}
