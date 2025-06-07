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

            fn operation_generator_with_loop_behavior(self, _: &Geometry, _: &DeviceFilter, segment: Segment, transition_mode: Option<TransitionMode>, loop_behavior: LoopBehavior) -> Result<Self::G, Self::Error> {
                let config = <Self as Modulation>::sampling_config(&self);
                let g = self.calc()?;
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

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let ext = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > #name #ty_generics #where_clause {
            /// Returns the expected radiation pressure. The value are normalized to 0-1.
            pub fn expected_radiation_pressure(self) -> Result<f32, ModulationError> {
                <Self as Modulation>::calc(self).map(|buf| {
                    1. / buf.len() as f32
                        * buf
                            .into_iter()
                            .map(|p| p as f32 / u8::MAX as f32)
                            .map(|p| p * p)
                            .sum::<f32>()
                })
            }
        }
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let inspect = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > Inspectable for #name #ty_generics #where_clause {
            type Result = ModulationInspectionResult;

            fn inspect(self, geometry: &Geometry, filter: &DeviceFilter) -> Result<InspectionResult<<Self as Inspectable>::Result>, <Self as Datagram>::Error> {
                let sampling_config = self.sampling_config();
                sampling_config.divide()?;
                let data = self.calc()?;
                let loop_behavior = LoopBehavior::Infinite;
                let segment = Segment::S0;
                let transition_mode = None;
                Ok(InspectionResult::new(
                    geometry,
                    filter,
                    |_| ModulationInspectionResult {
                            name: tynm::type_name::<Self>().to_string(),
                            data: data.clone(),
                            config: sampling_config,
                            loop_behavior,
                            segment,
                            transition_mode,
                    }
                ))
            }
        }
    };

    let generator = quote! {
        #datagram

        #ext

        #inspect
    };
    generator.into()
}
