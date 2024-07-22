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
            impl <#(#linetimes,)* #(#type_params,)*> IntoGainTransform<Self> for #name #ty_generics #where_clause {
                fn with_transform<GainTransformFT: Fn(&Transducer, Drive) -> Drive + Send + Sync, GainTransformF: Fn(&Device) -> GainTransformFT>(self, f: GainTransformF) -> GainTransform<Self, GainTransformFT, GainTransformF> {
                    GainTransform::new(self, f)
                }
            }
        }
    };

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

            #[tracing::instrument(skip(self, geometry))]
            // GRCOV_EXCL_START
            fn trace(&self, geometry: &Geometry) {
                <Self as Gain>::trace(self, geometry);
                if tracing::enabled!(tracing::Level::DEBUG) {
                    if let Ok(f) = <Self as Gain>::calc(self, geometry) {
                        geometry.devices().for_each(|dev| {
                            let f = f(dev);
                            if tracing::enabled!(tracing::Level::TRACE) {
                                tracing::debug!(
                                    "Device[{}]: {}",
                                    dev.idx(),
                                    dev.iter().map(|tr| f(tr)).join(", ")
                                );
                            } else {
                                tracing::debug!(
                                    "Device[{}]: {}, ..., {}",
                                    dev.idx(),
                                    f(&dev[0]),
                                    f(&dev[dev.num_transducers() - 1])
                                );
                            }
                        });
                    } else {
                        tracing::error!("Failed to calculate gain");
                    }
                }
            }
            // GRCOV_EXCL_STOP
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

            #[tracing::instrument(skip(self, geometry))]
            // GRCOV_EXCL_START
            fn trace(&self, geometry: &Geometry) {
                <Self as Gain>::trace(self, geometry);
                if tracing::enabled!(tracing::Level::DEBUG) {
                    if let Ok(f) = <Self as Gain>::calc(self, geometry) {
                        geometry.devices().for_each(|dev| {
                            let f = f(dev);
                            if tracing::enabled!(tracing::Level::TRACE) {
                                tracing::debug!(
                                    "Device[{}]: {}",
                                    dev.idx(),
                                    dev.iter().map(|tr| f(tr)).join(", ")
                                );
                            } else {
                                tracing::debug!(
                                    "Device[{}]: {}, ..., {}",
                                    dev.idx(),
                                    f(&dev[0]),
                                    f(&dev[dev.num_transducers() - 1])
                                );
                            }
                        });
                    } else {
                        tracing::error!("Failed to calculate gain");
                    }
                }
            }
            // GRCOV_EXCL_STOP
        }
    };

    let generator = quote! {
        #datagram

        #datagram_with_segment

        #cache

        #transform
    };
    generator.into()
}
