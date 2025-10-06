use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn impl_mod_macro(input: syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let datagram = quote! {
        impl <#(#lifetimes,)* #(#type_params,)*> DatagramL<'_> for #name #ty_generics #where_clause {
            type G = ModulationOperationGenerator;
            type Error = ModulationError;

            fn operation_generator_with_finite_loop(self, _: &Geometry, _: &Environment, _: &DeviceMask, segment: Segment, transition_params: transition_mode::TransitionModeParams, rep: u16) -> Result<Self::G, Self::Error> {
                let config = <Self as Modulation>::sampling_config(&self);
                let g = self.calc()?;
                Ok(Self::G {
                    g: std::sync::Arc::new(g),
                    config,
                    rep,
                    segment,
                    transition_params,
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
    let inspect = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > Inspectable<'_> for #name #ty_generics #where_clause {
            type Result = ModulationInspectionResult;

            fn inspect(
                self,
                geometry: &Geometry,
                _: &Environment,
                filter: &DeviceMask,
            ) -> Result<InspectionResult<Self::Result>, ModulationError> {
                let sampling_config = self.sampling_config();
                sampling_config.divide()?;
                let data = self.calc()?;
                Ok(InspectionResult::new(
                    geometry,
                    filter,
                    |_| ModulationInspectionResult {
                            data: data.clone(),
                            config: sampling_config,
                    }
                ))
            }
        }
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let segment_immediate = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasSegment<transition_mode::Immediate> for #name #ty_generics #where_clause {}
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let segment_ext = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasSegment<transition_mode::Ext> for #name #ty_generics #where_clause {}
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let segment_later = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasSegment<transition_mode::Later> for #name #ty_generics #where_clause {}
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let finite_loop_syncidx = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasFiniteLoop<transition_mode::SyncIdx> for #name #ty_generics #where_clause {}
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let finite_loop_systime = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasFiniteLoop<transition_mode::SysTime> for #name #ty_generics #where_clause {}
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let finite_loop_gpio = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasFiniteLoop<transition_mode::GPIO> for #name #ty_generics #where_clause {}
    };

    let lifetimes = generics.lifetimes();
    let type_params = generics.type_params();
    let (_, ty_generics, where_clause) = generics.split_for_impl();
    let finite_loop_later = quote! {
        impl <#(#lifetimes,)* #(#type_params,)* > internal::HasFiniteLoop<transition_mode::Later> for #name #ty_generics #where_clause {}
    };

    let generator = quote! {
        #datagram

        #inspect

        #segment_immediate

        #segment_ext

        #segment_later

        #finite_loop_syncidx

        #finite_loop_systime

        #finite_loop_gpio

        #finite_loop_later
    };
    generator.into()
}
