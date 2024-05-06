use proc_macro::TokenStream;
use quote::quote;
use syn::Meta;

pub(crate) fn impl_mod_macro(input: syn::DeriveInput) -> TokenStream {
    let attrs = &input.attrs;
    let name = &input.ident;
    let generics = &input.generics;

    let freq_div_no_change =
        if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data.clone() {
            fields.iter().any(|field| {
                let is_config = field
                    .ident
                    .as_ref()
                    .map(|ident| ident == "config")
                    .unwrap_or(false);
                let no_change = field.attrs.iter().any(
                    |attr| matches!(&attr.meta, Meta::Path(path) if path.is_ident("no_change")),
                );
                is_config && no_change
            })
        } else {
            false
        };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let freq_config = if freq_div_no_change {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
                /// Set sampling configuration
                ///
                /// # Arguments
                ///
                /// * `config` - Sampling configuration
                ///
                #[allow(clippy::needless_update)]
                pub fn with_sampling_config(self, config: SamplingConfiguration) -> Self {
                    Self {config, ..self}
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let loop_behavior = quote! {
            impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
                /// Set loop behavior
                ///
                /// # Arguments
                ///
                /// * `loop_behavior` - Loop behavior
                ///
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
            fn sampling_config(&self) -> SamplingConfiguration {
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
    let datagram = quote! {
        impl <#(#linetimes,)* #(#type_params,)* > DatagramS for #name #ty_generics #where_clause {
            type O1 = ModulationOp<Self>;
            type O2 = NullOp;

            fn operation_with_segment(self, segment: Segment, transition_mode: Option<TransitionMode>) -> (Self::O1, Self::O2) {
                (Self::O1::new(self, segment, transition_mode), Self::O2::default())
            }

            fn timeout(&self) -> Option<std::time::Duration> {
                Some(DEFAULT_TIMEOUT)
            }
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
                fn with_transform<ModulationTransformF: Fn(&Device, usize, u8) -> u8>(self, f: ModulationTransformF) -> ModulationTransform<Self, ModulationTransformF> {
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

    let gen = quote! {
        #prop

        #loop_behavior

        #freq_config

        #datagram

        #transform

        #cache

        #radiation_pressure
    };
    gen.into()
}
