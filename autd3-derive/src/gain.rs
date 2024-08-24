use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram = quote! {
        impl <#(#linetimes,)* #(#type_params,)*> Datagram for #name #ty_generics #where_clause
        {
            type G =  GainOperationGenerator<Self>;

            fn operation_generator(self, geometry: &Geometry) -> Result<Self::G, AUTDInternalError> {
                Self::G::new(
                    self,
                    geometry,
                    Segment::S0,
                    true,
                )
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram_with_segment = quote! {
        impl <#(#linetimes,)* #(#type_params,)*> DatagramS for #name #ty_generics #where_clause
        {
            type G =  GainOperationGenerator<Self>;

            fn operation_generator_with_segment(self, geometry: &Geometry, segment: Segment, transition: bool) -> Result<Self::G, AUTDInternalError> {
                Self::G::new(
                    self,
                    geometry,
                    segment,
                    transition,
                )
            }
        }
    };

    let generator = quote! {
        #datagram

        #datagram_with_segment
    };
    generator.into()
}
