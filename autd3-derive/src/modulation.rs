use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_mod_macro(input: syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let datagram = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > DatagramL for #name #ty_generics #where_clause {
            type G = ModulationOperationGenerator;
            type Error = ModulationError;

            fn operation_generator_with_loop_behavior(self, _: &Geometry, _: bool, segment: Segment, transition_mode: Option<TransitionMode>, loop_behavior: LoopBehavior) -> Result<Self::G, Self::Error> {
                let config = <Self as Modulation>::sampling_config(&self)?;
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

            fn option(&self) -> DatagramOption {
                DatagramOption::default()
            }
        }
    };

    let generator = quote! {
        #datagram
    };
    generator.into()
}
