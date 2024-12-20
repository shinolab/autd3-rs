use proc_macro::TokenStream;
use quote::quote;
use syn::Meta;

pub(crate) fn impl_mod_macro(input: syn::DeriveInput) -> TokenStream {
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
                /// Set the sampling configuration.
                pub fn with_sampling_config<TryIntoSamplingConfig: TryInto<SamplingConfig>>(mut self, config: TryIntoSamplingConfig) -> Result<Self, TryIntoSamplingConfig::Error>
                {
                    self.config = config.try_into()?;
                    Ok(self)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let loop_behavior = quote! {
            impl <#(#linetimes,)* #(#type_params,)*> #name #ty_generics #where_clause {
                /// Set the loop behavior.
                #[must_use]
                pub fn with_loop_behavior(mut self, loop_behavior: LoopBehavior) -> Self {
                    self.loop_behavior = loop_behavior;
                    self
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
        impl <#(#linetimes,)* #(#type_params,)* > DatagramS for #name #ty_generics #where_clause {
            type G =  ModulationOperationGenerator;

            fn operation_generator_with_segment(self, _: &Geometry, segment: Segment, transition_mode: Option<TransitionMode>) -> Result<Self::G, AUTDDriverError> {
                let config = self.sampling_config();
                let loop_behavior = self.loop_behavior();
                let g = self.calc()?;
                tracing::trace!("Modulation buffer: {:?}", g);
                Ok(Self::G {
                    g: std::sync::Arc::new(g),
                    config,
                    loop_behavior,
                    segment,
                    transition_mode,
                })
            }
        }
    };

    let generator = quote! {
        #prop

        #loop_behavior

        #sampling_config

        #datagram_with_segment_transition

    };
    generator.into()
}
