use proc_macro::TokenStream;
use quote::quote;
use syn::Meta;

pub(crate) fn impl_mod_macro(input: syn::DeriveInput) -> TokenStream {
    let attrs = &input.attrs;
    let name = &input.ident;
    let generics = &input.generics;

    let no_change = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data.clone() {
        fields.iter().any(|field| {
            let is_config = field
                .ident
                .as_ref()
                .map(|ident| ident == "config")
                .unwrap_or(false);
            let no_change = field
                .attrs
                .iter()
                .any(|attr| matches!(&attr.meta, Meta::Path(path) if path.is_ident("no_change")));
            is_config && no_change
        })
    } else {
        false
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let sampling_config = if no_change {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
                #[allow(clippy::needless_update)]
                pub fn with_sampling_config(self, config: impl Into<SamplingConfig>) -> Self {
                    Self {config: config.into(), ..self}
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let loop_behavior = quote! {
            impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
                #[allow(clippy::needless_update)]
                pub fn with_loop_behavior(self, loop_behavior: LoopBehavior) -> Self {
                    Self {loop_behavior, ..self}
                }
            }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let prop = quote! {
            impl <#(#linetimes,)* #(#type_params,)*> ModulationProperty for #name #ty_generics #where_clause {
                fn sampling_config(&self) -> SamplingConfig {
                    self.config
                }

                fn loop_behavior(&self) -> LoopBehavior {
                    self.loop_behavior
                }
            }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let datagram_with_segment_transition = quote! {
        impl <#(#linetimes,)* #(#type_params,)* > DatagramST for #name #ty_generics #where_clause {
            type G =  ModulationOperationGenerator;

            fn operation_generator_with_segment(self, _: &Geometry, segment: Segment, transition_mode: Option<TransitionMode>) -> Result<Self::G, AUTDInternalError> {
                Ok(Self::G {
                    g: std::sync::Arc::new(self.calc()?),
                    config: self.sampling_config(),
                    loop_behavior: self.loop_behavior(),
                    segment,
                    transition_mode,
                })
            }

            fn timeout(&self) -> Option<std::time::Duration> {
                Some(DEFAULT_TIMEOUT)
            }

            fn parallel_threshold(&self) -> Option<usize> {
                Some(usize::MAX)
            }

            #[tracing::instrument(skip(self, geometry))]
            // GRCOV_EXCL_START
            fn trace(&self, geometry: &Geometry) {
                <Self as Modulation>::trace(self, geometry);
                if tracing::enabled!(tracing::Level::DEBUG) {
                    if let Ok(buf) = <Self as Modulation>::calc(self) {
                        match buf.len() {
                            0 => {
                                tracing::error!("Buffer is empty");
                                return;
                            }
                            1 => {
                                tracing::debug!("Buffer: {:#04X}", buf[0]);
                            }
                            2 => {
                                tracing::debug!("Buffer: {:#04X}, {:#04X}", buf[0], buf[1]);
                            }
                            _ => {
                                if tracing::enabled!(tracing::Level::TRACE) {
                                    tracing::debug!(
                                        "Buffer: {}",
                                        buf.iter()
                                            .format_with(", ", |elt, f| f(&format_args!("{:#04X}", elt)))
                                    );
                                } else {
                                    tracing::debug!(
                                        "Buffer: {:#04X}, ..., {:#04X} ({})",
                                        buf[0],
                                        buf[buf.len() - 1],
                                        buf.len()
                                    );
                                }
                            }
                        }
                    } else {
                        tracing::error!("Failed to calculate modulation");
                    }
                }
            }
            // GRCOV_EXCL_STOP
        }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let transform = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_modulation_transform"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoModulationTransform<Self> for #name #ty_generics #where_clause {
                fn with_transform<ModulationTransformF: Fn(usize, u8) -> u8>(self, f: ModulationTransformF) -> ModulationTransform<Self, ModulationTransformF> {
                    ModulationTransform::new(self, f)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let cache = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_modulation_cache"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoModulationCache<Self> for #name #ty_generics #where_clause {
                fn with_cache(self) -> ModulationCache<Self> {
                    ModulationCache::new(self)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let radiation_pressure = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_radiation_pressure"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoRadiationPressure<Self> for #name #ty_generics #where_clause {
                fn with_radiation_pressure(self) -> RadiationPressure<Self> {
                    RadiationPressure::new(self)
                }
            }
        }
    };

    let generator = quote! {
        #prop

        #loop_behavior

        #sampling_config

        #datagram_with_segment_transition

        #transform

        #cache

        #radiation_pressure
    };
    generator.into()
}
