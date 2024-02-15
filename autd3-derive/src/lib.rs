use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Meta, WhereClause};

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
    let input = parse_macro_input!(input as syn::DeriveInput);

    let attrs = &input.attrs;
    let name = &input.ident;
    let generics = &input.generics;

    let freq_div_no_change = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data {
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
            type O1 = ModulationOp;
            type O2 = NullOp;

            fn operation_with_segment(self, segment: Segment, update_segment: bool) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
                let freq_div = self.config.frequency_division();
                Ok((Self::O1::new(self.calc()?, freq_div, self.loop_behavior, segment, update_segment), Self::O2::default()))
            }

            fn timeout(&self) -> Option<std::time::Duration> {
                Some(std::time::Duration::from_millis(200))
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
                fn with_transform<ModulationTransformF: Fn(usize, EmitIntensity) -> EmitIntensity>(self, f: ModulationTransformF) -> ModulationTransform<Self, ModulationTransformF> {
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

#[proc_macro_derive(Gain, attributes(no_gain_cache, no_gain_transform))]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_gain_macro(ast)
}

fn to_gain_where(where_clause: Option<&WhereClause>) -> proc_macro2::TokenStream {
    match where_clause {
        Some(where_clause) => {
            let where_predicate_punctuated_list = where_clause
                .predicates
                .iter()
                .map(|where_predicate| match where_predicate {
                    syn::WherePredicate::Type(_) => {
                        quote! { #where_predicate }
                    }
                    _ => quote! {},
                })
                .collect::<Vec<_>>();
            quote! { where GainOp<Self>: Operation, #(#where_predicate_punctuated_list),* }
        }
        None => {
            quote! { where GainOp<Self>: Operation }
        }
    }
}

fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let attrs = &ast.attrs;
    let name = &ast.ident;
    let generics = &ast.generics;

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let where_clause = to_gain_where(where_clause);
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
    let where_clause = to_gain_where(where_clause);
    let transform = if attrs
        .iter()
        .any(|attr| attr.path().is_ident("no_gain_transform"))
    {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes,)* #(#type_params,)*> IntoGainTransform<Self> for #name #ty_generics #where_clause {
                fn with_transform<GainTransformF: Fn(&Device, &Transducer, Drive) -> Drive>(self, f: GainTransformF) -> GainTransform<Self, GainTransformF> {
                    GainTransform::new(self, f)
                }
            }
        }
    };

    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let where_clause = to_gain_where(where_clause);
    let gen = quote! {
        impl <#(#linetimes,)* #(#type_params,)*> DatagramS for #name #ty_generics #where_clause
        {
            type O1 = GainOp<Self>;
            type O2 = NullOp;

            fn operation_with_segment(self, segment: Segment, update_segment: bool) -> Result<(Self::O1, Self::O2), AUTDInternalError> {
                Ok((Self::O1::new(segment, update_segment, self), Self::O2::default()))
            }
        }

        #cache

        #transform
    };
    gen.into()
}
