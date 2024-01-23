use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Meta};

#[proc_macro_derive(Modulation, attributes(no_change))]
pub fn modulation_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

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

    let name = &input.ident;
    let generics = &input.generics;
    let linetimes_prop = generics.lifetimes();
    let linetimes_impl = generics.lifetimes();
    let linetimes_datagram = generics.lifetimes();
    let type_params_prop = generics.type_params();
    let type_params_impl = generics.type_params();
    let type_params_datagram = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let freq_config = if freq_div_no_change {
        quote! {}
    } else {
        quote! {
            impl <#(#linetimes_impl,)* #(#type_params_impl,)*> #name #ty_generics #where_clause {
                /// Set sampling configuration
                ///
                /// # Arguments
                ///
                /// * `config` - Sampling configuration
                ///
                #[allow(clippy::needless_update)]
                pub fn with_sampling_config(self, config: autd3_driver::common::SamplingConfiguration) -> Self {
                    Self {config, ..self}
                }
            }
        }
    };

    let gen = quote! {
        impl <#(#linetimes_prop,)* #(#type_params_prop,)*> ModulationProperty for #name #ty_generics #where_clause {
            fn sampling_config(&self) -> autd3_driver::common::SamplingConfiguration {
                self.config
            }
        }

        #freq_config

        impl <#(#linetimes_datagram,)* #(#type_params_datagram,)* > Datagram for #name #ty_generics #where_clause {
            type O1 = ModulationOp;
            type O2 = NullOp;

            fn operation(self) -> Result<(Self::O1, Self::O2), autd3_driver::error::AUTDInternalError> {
                let freq_div = self.config.frequency_division();
                Ok((Self::O1::new(self.calc()?, freq_div), Self::O2::default()))
            }

            fn timeout(&self) -> Option<std::time::Duration> {
                Some(std::time::Duration::from_millis(200))
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Gain)]
pub fn gain_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_gain_macro(ast)
}

fn impl_gain_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let linetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let where_clause = match where_clause {
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
    };

    let gen = quote! {
        impl <#(#linetimes,)* #(#type_params,)*> Datagram for #name #ty_generics #where_clause
        {
            type O1 = GainOp<Self>;
            type O2 = NullOp;

            fn operation(self) -> Result<(Self::O1, Self::O2), autd3_driver::error::AUTDInternalError> {
                Ok((Self::O1::new(self), Self::O2::default()))
            }
        }
    };
    gen.into()
}
