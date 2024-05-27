use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let attrs = &ast.attrs;
    let name = &ast.ident;
    let generics = &ast.generics;

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let cache = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_gain_cache"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoGainCache<Self> for #name #ty_generics #where_clause {
                fn with_cache(self) -> GainCache<Self> {
                    GainCache::new(self)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let transform = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_gain_transform"))
    {
        quote! {}
    } else {
        quote! {
            impl <'autd3, #(#linetimes,)* #(#type_params,)*> IntoGainTransform<Self> for #name #ty_generics #where_clause {
                fn with_transform<GainTransformFT: Fn(&Transducer, Drive) -> Drive + Send + Sync, GainTransformF: Fn(&Device) -> GainTransformFT + Send + Sync + Clone>(self, f: GainTransformF) -> GainTransform<Self, GainTransformFT, GainTransformF> {
                    GainTransform::new(self, f)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram = quote! {
        impl <'autd3, #(#linetimes,)* #(#type_params,)*> Datagram<'autd3> for #name #ty_generics #where_clause
        {
            type O1 = GainOp;
            type O2 = NullOp;
            type G =  GainOperationGenerator<'autd3>;

            fn operation_generator(self, geometry: &'autd3 Geometry) -> Result<Self::G, AUTDInternalError> {
                let g = self.calc(geometry)?;
                Ok(Self::G {
                    g: Box::new(g),
                    segment: Segment::S0,
                    transition: true,
                })
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram_with_segment = quote! {
        impl <'autd3, #(#linetimes,)* #(#type_params,)*> DatagramS<'autd3> for #name #ty_generics #where_clause
        {
            type O1 = GainOp;
            type O2 = NullOp;
            type G =  GainOperationGenerator<'autd3>;

            fn operation_generator_with_segment(self, geometry: &'autd3 Geometry, segment: Segment, transition: bool) -> Result<Self::G, AUTDInternalError> {
                let g = self.calc(geometry)?;
                Ok(Self::G {
                    g: Box::new(g),
                    segment,
                    transition,
                })
            }
        }
    };

    let gen = quote! {
        #datagram

        #datagram_with_segment

        #cache

        #transform
    };
    gen.into()
}
