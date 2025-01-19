use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_mod_option_macro(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let gain_option = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> ModulationOption for #name #ty_generics #where_clause
        {
            fn segment(&self) -> Segment {
                self.segment
            }

            fn transition_mode(&self) -> Option<TransitionMode> {
                self.transition_mode
            }

            fn sampling_config(&self) -> SamplingConfig {
                self.sampling_config
            }

            fn loop_behavior(&self) -> LoopBehavior {
                self.loop_behavior
            }
        }
    };

    let generator = quote! {
        #gain_option
    };
    generator.into()
}

pub(crate) fn impl_mod_macro(input: syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let lifetimes = generics.lifetimes();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let type_params = generics.type_params();
    let option = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> GetModulationOption for #name #ty_generics #where_clause
        {
            fn option(&self) -> &<Self as Modulation>::Option {
                &self.option
            }
        }
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let datagram = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > Datagram for #name #ty_generics #where_clause {
            type G = ModulationOperationGenerator;
            type Error = ModulationError;

            fn operation_generator(self, _: &Geometry) -> Result<Self::G, Self::Error> {
                let option = <Self as GetModulationOption>::option(&self);
                let segment = option.segment();
                let transition_mode = option.transition_mode();
                let config = option.sampling_config();
                let loop_behavior = option.loop_behavior();
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
        #datagram

        #option
    };
    generator.into()
}
