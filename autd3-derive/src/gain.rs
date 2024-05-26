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
                fn with_transform<GainTransformFT: Fn(&Transducer, Drive) -> Drive , GainTransformF: Fn(&Device) -> GainTransformFT + Send + Sync>(self, f: GainTransformF) -> GainTransform<Self, GainTransformFT, GainTransformF> {
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
            type O1 = GainOp<'autd3>;
            type O2 = NullOp;

            fn operation(&'autd3 self, geometry: &'autd3 Geometry) -> Result<impl Fn(&'autd3 Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError> {
                let f = self.calc(geometry, GainFilter::All)?;
                Ok(move |dev| (GainOp::new(Segment::S0, true, f(dev)), NullOp::default()))
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let datagram_with_segment = quote! {
        impl <'autd3, #(#linetimes,)* #(#type_params,)*> DatagramS<'autd3> for #name #ty_generics #where_clause
        {
            type O1 = GainOp<'autd3>;
            type O2 = NullOp;

            fn operation_with_segment(&'autd3 self, geometry: &'autd3 Geometry, segment: Segment, transition: bool) -> Result<impl Fn(&'autd3 Device) -> (Self::O1, Self::O2) + Send + Sync, AUTDInternalError>  {
                let f = self.calc(geometry, GainFilter::All)?;
                Ok(move |dev| (GainOp::new(segment, transition, f(dev)), NullOp::default()))
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
